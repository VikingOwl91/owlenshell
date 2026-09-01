use crate::domain::ids::{HostId, OutputId};
use crate::domain::placement::{Anchor, PlacementDecision, SavedPlacement};
use crate::instrumentation;
use crate::model::SurfaceKind;
use crate::persistence::Placements;
use crate::placement::{
    margins_from_offset, HostPlacementConstraints, PlacementContext, PlacementEngine, MIN_VISIBLE,
};
use crate::platform::{
    OutputManager, PlatformLifecycleEvent, PlatformPointerEvent, RenderBridge, SurfaceManager,
    TopologyGeneration, TopologySnapshot, WaylandPlatform,
};
use crate::runtime::poc_compat::{
    compute_attach_geometry, is_bar_host, is_calendar_host, surface_spec_for, HANDLE_HEIGHT,
};
use slint::platform::{PointerEventButton, WindowAdapter};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

/// Logical state tracked per shell host within the Coordinator.
///
/// Contains logical visibility, optional user-saved placement intent,
/// current placement decision, and the topology generation against which
/// that decision was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalHostState {
    pub host_id: HostId,
    pub kind: SurfaceKind,
    pub visible: bool,
    pub saved_placement: Option<SavedPlacement>,
    pub current_decision: Option<PlacementDecision>,
    pub decision_generation: Option<TopologyGeneration>,
}

impl LogicalHostState {
    pub fn new(
        host_id: HostId,
        kind: SurfaceKind,
        saved_placement: Option<SavedPlacement>,
    ) -> Self {
        Self {
            host_id,
            kind,
            visible: true,
            saved_placement,
            current_decision: None,
            decision_generation: None,
        }
    }
}

/// Active user drag interaction state, keyed by logical `HostId`.
#[derive(Debug, Clone, PartialEq)]
pub struct DragState {
    pub host_id: HostId,
    pub grab: (f64, f64),
    pub start_margins: (i32, i32, i32, i32),
    pub anchor: Anchor,
    pub output_id: OutputId,
}

/// The Thin Coordinator orchestrating logical host reconciliation,
/// placement decisions, persistence, and typed platform commands.
///
/// Holds zero Wayland/SCTK protocol types.
pub struct Coordinator {
    hosts: RefCell<BTreeMap<HostId, LogicalHostState>>,
    surface_manager: Rc<RefCell<SurfaceManager>>,
    render_bridge: Rc<RenderBridge>,
    output_manager: Rc<RefCell<OutputManager>>,
    store: RefCell<Placements>,
    drag: RefCell<Option<DragState>>,
    outputs_ready: Cell<bool>,
    exit: Cell<bool>,
    first_frame: Cell<bool>,
}

impl Coordinator {
    /// Creates a new Coordinator instance.
    pub fn new(
        output_manager: Rc<RefCell<OutputManager>>,
        surface_manager: Rc<RefCell<SurfaceManager>>,
        render_bridge: Rc<RenderBridge>,
        store: Placements,
    ) -> Self {
        Self {
            hosts: RefCell::new(BTreeMap::new()),
            surface_manager,
            render_bridge,
            output_manager,
            store: RefCell::new(store),
            drag: RefCell::new(None),
            outputs_ready: Cell::new(false),
            exit: Cell::new(false),
            first_frame: Cell::new(false),
        }
    }

    /// Primary production API: Attaches or updates a logical host.
    pub fn attach_host(
        &self,
        host_id: HostId,
        kind: SurfaceKind,
        saved_placement: Option<SavedPlacement>,
    ) {
        let mut hosts = self.hosts.borrow_mut();
        if let Some(host) = hosts.get_mut(&host_id) {
            host.kind = kind;
            host.saved_placement = saved_placement;
            host.visible = true;
            // Invalidate current decision to force re-resolution
            host.current_decision = None;
            host.decision_generation = None;
        } else {
            hosts.insert(
                host_id.clone(),
                LogicalHostState::new(host_id, kind, saved_placement),
            );
        }
    }

    /// Primary production API: Hides a logical host and cancels active drag if targeting it.
    pub fn hide_host(&self, host_id: &HostId) {
        let mut hosts = self.hosts.borrow_mut();
        if let Some(host) = hosts.get_mut(host_id) {
            host.visible = false;
            self.surface_manager.borrow_mut().retire(host_id);
            let dragging_this = self
                .drag
                .borrow()
                .as_ref()
                .is_some_and(|d| &d.host_id == host_id);
            if dragging_this {
                self.drag.borrow_mut().take();
            }
        }
    }

    /// Primary production API: Hides all hosts matching a given SurfaceKind.
    pub fn hide_kind(&self, kind: SurfaceKind) {
        let mut hosts = self.hosts.borrow_mut();
        for (host_id, host) in hosts.iter_mut().filter(|(_, h)| h.kind == kind) {
            host.visible = false;
            self.surface_manager.borrow_mut().retire(host_id);
            let dragging_this = self
                .drag
                .borrow()
                .as_ref()
                .is_some_and(|d| &d.host_id == host_id);
            if dragging_this {
                self.drag.borrow_mut().take();
            }
        }
    }

    /// Returns whether the shell application has been requested to exit.
    pub fn should_exit(&self) -> bool {
        self.exit.get()
    }

    /// Sets the shell application exit request.
    pub fn request_exit(&self) {
        self.exit.set(true);
    }

    /// Returns a copy of the active drag state, if any.
    pub fn drag_state(&self) -> Option<DragState> {
        self.drag.borrow().clone()
    }

    /// Sets the active drag state directly (used in tests and input routing).
    pub fn set_drag_state(&self, drag: Option<DragState>) {
        *self.drag.borrow_mut() = drag;
    }

    /// Returns whether the first frame milestone has fired.
    pub fn first_frame_fired(&self) -> bool {
        self.first_frame.get()
    }

    /// Sets the first frame milestone flag (used in tests).
    pub fn set_first_frame_fired(&self, fired: bool) {
        self.first_frame.set(fired);
    }

    /// Returns a snapshot of a host's logical state.
    pub fn get_host(&self, host_id: &HostId) -> Option<LogicalHostState> {
        self.hosts.borrow().get(host_id).cloned()
    }

    /// Returns a clone of the placements store.
    pub fn placements_store(&self) -> Placements {
        self.store.borrow().clone()
    }

    /// Cancels active drag if the target output does not exist in the given topology snapshot.
    pub fn cancel_drag_if_output_missing(&self, snapshot: &TopologySnapshot) -> bool {
        let missing = self
            .drag
            .borrow()
            .as_ref()
            .is_some_and(|drag| snapshot.get(drag.output_id).is_none());
        if missing {
            self.drag.borrow_mut().take();
            true
        } else {
            false
        }
    }

    /// Processes a typed platform lifecycle event.
    pub fn process_lifecycle_event(&self, event: PlatformLifecycleEvent) {
        match event {
            PlatformLifecycleEvent::FrameRendered { host_id, .. } => {
                let hosts = self.hosts.borrow();
                if let Some(host) = hosts.get(&host_id) {
                    if is_bar_host(host.kind) && !self.first_frame.replace(true) {
                        instrumentation::milestone("first-frame");
                        instrumentation::milestone("shell-ready");
                    }
                }
            }
            PlatformLifecycleEvent::Closed { host_id, .. } => {
                let hosts = self.hosts.borrow();
                if let Some(host) = hosts.get(&host_id) {
                    // Bar close requests application exit
                    if is_bar_host(host.kind) {
                        self.exit.set(true);
                    }
                    // Native surface close cancels active drag targeting that surface
                    let dragging_this = self
                        .drag
                        .borrow()
                        .as_ref()
                        .is_some_and(|drag| drag.host_id == host_id);
                    if dragging_this {
                        self.drag.borrow_mut().take();
                    }
                }
            }
        }
    }

    /// Builds current placement context from active topology and bar placement.
    pub fn resolve_ctx(&self) -> PlacementContext {
        let spawn_output_id = {
            let hosts = self.hosts.borrow();
            hosts.values().find_map(|h| {
                if is_bar_host(h.kind) {
                    self.surface_manager.borrow().output_id_for_host(&h.host_id)
                } else {
                    None
                }
            })
        };
        PlacementContext {
            spawn_output_id,
            preferred_name: self.output_manager.borrow().preferred_output_name(),
            focused_output_id: None,
        }
    }

    /// Pure logical reconciliation:
    /// 1. Reads current topology snapshot and generation.
    /// 2. Iterates over visible hosts.
    /// 3. If decision is missing or resolved against a stale topology generation, re-resolves
    ///    via `PlacementEngine::resolve(saved.as_ref(), ...)`.
    /// 4. Compares target output with current native instance. If mismatched, retires stale native instance
    ///    and requests redraw.
    pub fn reconcile(&self) {
        let output_manager = self.output_manager.borrow();
        let snapshot = output_manager.snapshot();
        let current_gen = snapshot.generation();
        let placement_candidates: Vec<_> = snapshot.placement_capable_outputs().cloned().collect();
        drop(output_manager);

        let ctx = self.resolve_ctx();
        let constraints = HostPlacementConstraints::default();

        let host_keys: Vec<HostId> = self.hosts.borrow().keys().cloned().collect();

        for host_id in host_keys {
            let (kind, visible, saved) = {
                let hosts = self.hosts.borrow();
                let Some(host) = hosts.get(&host_id) else {
                    continue;
                };
                (host.kind, host.visible, host.saved_placement.clone())
            };

            if !visible {
                continue;
            }

            // Check if existing decision is valid and matching current topology generation
            let needs_resolution = {
                let hosts = self.hosts.borrow();
                let host = hosts.get(&host_id).unwrap();
                host.current_decision.is_none() || host.decision_generation != Some(current_gen)
            };

            let decision = if needs_resolution {
                let dec = PlacementEngine::resolve(
                    saved.as_ref(),
                    &placement_candidates,
                    &ctx,
                    &constraints,
                );
                let target = dec.target_output_id();
                let target_name = target.and_then(|id| {
                    placement_candidates
                        .iter()
                        .find(|o| o.id == id)
                        .and_then(|o| o.name.clone())
                });

                instrumentation::placement_decision(
                    match kind {
                        SurfaceKind::Bar => "bar",
                        SurfaceKind::Calendar => "calendar",
                    },
                    format!("{:?}", dec.reason()).as_str(),
                    target_name.as_deref(),
                    dec.is_fallback(),
                );

                let mut hosts = self.hosts.borrow_mut();
                if let Some(host) = hosts.get_mut(&host_id) {
                    host.current_decision = Some(dec.clone());
                    host.decision_generation = Some(current_gen);
                }
                dec
            } else {
                let hosts = self.hosts.borrow();
                hosts
                    .get(&host_id)
                    .unwrap()
                    .current_decision
                    .clone()
                    .unwrap()
            };

            let current_output = self.surface_manager.borrow().output_id_for_host(&host_id);
            let target_output = decision.target_output_id();

            if target_output.is_none() {
                self.surface_manager.borrow_mut().retire(&host_id);
                continue;
            }

            if current_output != target_output {
                self.surface_manager.borrow_mut().retire(&host_id);
                if let Some(adapter) = self.render_bridge.get_adapter(&host_id) {
                    WindowAdapter::request_redraw(&*adapter.window);
                }
            }
        }
    }

    /// Attaches or recreates a native surface instance on the platform if needed.
    fn try_attach_host(&self, platform: &mut WaylandPlatform, host_id: &HostId) {
        let (kind, visible, saved, decision) = {
            let hosts = self.hosts.borrow();
            let Some(host) = hosts.get(host_id) else {
                return;
            };
            (
                host.kind,
                host.visible,
                host.saved_placement.clone(),
                host.current_decision.clone(),
            )
        };

        if !visible {
            return;
        }

        let Some(adapter) = self.render_bridge.get_adapter(host_id) else {
            return;
        };

        let Some(decision) = decision else {
            return;
        };

        let Some(target_output_id) = decision.target_output_id() else {
            return;
        };

        let output_manager = self.output_manager.borrow();
        let snapshot = output_manager.snapshot();
        let Some(snap) = snapshot.get(target_output_id).cloned() else {
            return;
        };
        let Some(output) = output_manager.get_wl_output(target_output_id) else {
            return;
        };
        drop(output_manager);

        let preferred = adapter.preferred.get();
        let spec = surface_spec_for(kind);
        let geo = compute_attach_geometry(kind, saved.as_ref(), preferred, snap.logical_size);

        let params = crate::platform::NativeSurfaceParams {
            host_id: host_id.clone(),
            spec,
            output_id: target_output_id,
            output_scale: snap.scale,
            anchor: geo.anchor,
            margins: geo.margins,
            requested_size: geo.requested_size,
        };

        platform.attach_surface(params, &output);
        WindowAdapter::request_redraw(&*adapter.window);
    }

    /// Completes an active drag interaction:
    /// - Computes new commit via `PlacementEngine::commit_move`.
    /// - Persists updated placement to Placements store.
    /// - Reconfigures margins or retires surface for cross-output recreation.
    pub fn finish_drag(&self, drag: &DragState, pointer: (f64, f64)) {
        let host_id = drag.host_id.clone();
        let Some(adapter) = self.render_bridge.get_adapter(&host_id) else {
            return;
        };
        let pref = adapter.preferred.get();
        let widget = (pref.width.max(1.0) as i32, pref.height.max(1.0) as i32);
        let current_margins = self
            .surface_manager
            .borrow()
            .get_instance(&host_id)
            .map(|inst| inst.margins)
            .unwrap_or(drag.start_margins);

        let output_manager = self.output_manager.borrow();
        let snapshot = output_manager.snapshot();
        let outputs: Vec<_> = snapshot.placement_capable_outputs().cloned().collect();
        let commit = PlacementEngine::commit_move(
            pointer,
            drag.grab,
            widget,
            current_margins,
            drag.anchor,
            drag.output_id,
            &outputs,
            None,
            MIN_VISIBLE,
        );
        drop(output_manager);

        let Some(commit) = commit else {
            return;
        };
        let target_output_id = commit.target_output_id;
        let placement = commit.saved_placement;

        // Persist only on completed user drop
        {
            let mut store = self.store.borrow_mut();
            store.set(host_id.clone(), placement.clone());
            if let Err(err) = store.save() {
                tracing::warn!(%err, "failed to persist placement");
            }
        }

        let recreate = target_output_id != drag.output_id;
        {
            let mut hosts = self.hosts.borrow_mut();
            if let Some(host) = hosts.get_mut(&host_id) {
                host.saved_placement = Some(placement.clone());
                // Invalidate decision to force fresh reconciliation
                host.current_decision = None;
                host.decision_generation = None;
            }
        }

        if recreate {
            self.surface_manager.borrow_mut().retire(&host_id);
        } else {
            let new_margins = margins_from_offset(placement.anchor, placement.offset);
            self.surface_manager.borrow_mut().reconfigure_placement(
                &host_id,
                placement.anchor,
                new_margins,
            );
        }

        tracing::info!(
            output = placement.output.name.as_deref(),
            anchor = ?placement.anchor,
            offset = ?placement.offset,
            recreate,
            "surface placed via drag"
        );
    }

    /// Ordered Coordinator execution cycle:
    /// 1. Dispatches pending platform events.
    /// 2. Handles topology changes and updates outputs-ready milestone.
    /// 3. Drains platform lifecycle events.
    /// 4. Drains pointer input events for drag interactions.
    /// 5. Drains keyboard events.
    /// 6. Reconciles logical host state against current topology generation.
    /// 7. Attaches and renders visible surfaces.
    /// 8. Flushes platform requests.
    pub fn pump(&self, platform: &mut WaylandPlatform) -> Result<(), Box<dyn std::error::Error>> {
        let _ = platform.dispatch_pending()?;

        let is_topology_dirty = platform.is_topology_dirty();
        if is_topology_dirty {
            platform.clear_topology_dirty();
            platform.refresh_outputs();
            let has_outputs = !self.output_manager.borrow().snapshot().is_empty();
            if has_outputs && !self.outputs_ready.replace(true) {
                instrumentation::milestone("outputs-ready");
            }

            let output_manager = self.output_manager.borrow();
            let snapshot = output_manager.snapshot();
            self.cancel_drag_if_output_missing(&snapshot);
            drop(output_manager);
        }

        let pending_resolution = {
            let hosts = self.hosts.borrow();
            hosts
                .values()
                .any(|host| host.visible && host.current_decision.is_none())
        };

        if is_topology_dirty || pending_resolution {
            self.reconcile();
        }

        // Drain typed platform lifecycle events
        while let Some(lifecycle_ev) = platform.pop_lifecycle_event() {
            self.process_lifecycle_event(lifecycle_ev);
        }

        // Drain typed pointer input events for drag interactions
        while let Some(ptr_ev) = platform.pop_pointer_event() {
            match ptr_ev {
                PlatformPointerEvent::Press {
                    host_id,
                    position,
                    button: PointerEventButton::Left,
                } => {
                    let hosts = self.hosts.borrow();
                    if let Some(host) = hosts.get(&host_id) {
                        if is_calendar_host(host.kind) {
                            let mods = platform.modifiers();
                            let handle = position.1 < HANDLE_HEIGHT;
                            if (mods.ctrl && mods.alt) || handle {
                                if let Some(inst) =
                                    self.surface_manager.borrow().get_instance(&host_id)
                                {
                                    *self.drag.borrow_mut() = Some(DragState {
                                        host_id: host_id.clone(),
                                        grab: position,
                                        start_margins: inst.margins,
                                        anchor: inst.anchor,
                                        output_id: inst.output_id,
                                    });
                                }
                            }
                        }
                    }
                }
                PlatformPointerEvent::Release {
                    host_id,
                    position,
                    button: PointerEventButton::Left,
                } => {
                    if let Some(drag) = self.drag.borrow_mut().take() {
                        if drag.host_id == host_id {
                            self.finish_drag(&drag, position);
                        }
                    }
                }
                _ => {}
            }
        }

        // Drain typed keyboard events
        while let Some(_kb_ev) = platform.pop_keyboard_event() {}

        // Ensure visible surfaces are attached and rendered
        let host_keys: Vec<HostId> = self.hosts.borrow().keys().cloned().collect();
        for host_id in host_keys {
            let (visible, needs_attach) = {
                let hosts = self.hosts.borrow();
                let Some(host) = hosts.get(&host_id) else {
                    continue;
                };
                let has_native = self
                    .surface_manager
                    .borrow()
                    .has_native_instance(&host.host_id);
                (host.visible, !has_native && host.visible)
            };

            if needs_attach {
                self.try_attach_host(platform, &host_id);
            }
            if visible {
                platform.render_surface(&host_id);
            }
        }

        platform.flush()?;
        Ok(())
    }
}
