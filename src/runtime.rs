pub mod placement_engine;

use crate::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use crate::domain::ids::{HostId, OutputId};
use crate::instrumentation;
pub use crate::model::SurfaceKind;
use crate::model::{
    Anchor, OutputSnapshot, SavedPlacement, BAR_HEIGHT, CALENDAR_KEY, HANDLE_HEIGHT,
};
use crate::persistence::Placements;
use crate::placement::{
    clamp_offset, margins_from_offset, resolve, PlacementContext, PlacementEngine, MIN_VISIBLE,
};
use crate::platform::{
    OutputManager, OwlenWindowAdapter, PlatformLifecycleEvent, PlatformPointerEvent, RenderBridge,
    SurfaceManager, SurfaceSpec, WaylandPlatform,
};
use slint::platform::{PointerEventButton, WindowAdapter};
use slint::LogicalSize;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct AppShared {
    pub surfaces: RefCell<Vec<SurfaceState>>,
    pub surface_manager: Rc<RefCell<SurfaceManager>>,
    pub render_bridge: Rc<RenderBridge>,
    pub output_manager: Rc<RefCell<OutputManager>>,
    pub store: RefCell<Placements>,
    pub drag: RefCell<Option<DragState>>,
    pub outputs_ready: Cell<bool>,
    pub exit: Cell<bool>,
    pub first_frame: Cell<bool>,
}

pub struct SurfaceState {
    pub host_id: HostId,
    pub kind: SurfaceKind,
    pub adapter: Rc<OwlenWindowAdapter>,
    pub placement: Option<SavedPlacement>,
    pub visible: bool,
}

pub struct DragState {
    pub idx: usize,
    pub grab: (f64, f64),
    pub start_margins: (i32, i32, i32, i32),
    pub anchor: Anchor,
    pub output_id: OutputId,
}

impl SurfaceState {
    /// Prepares an existing surface state for reattachment or recreation with updated placement.
    pub fn prepare_recreation(
        &mut self,
        adapter: Rc<OwlenWindowAdapter>,
        placement: Option<SavedPlacement>,
    ) {
        self.adapter = adapter;
        self.placement = placement;
        self.visible = true;
    }
}

pub fn attach_host(
    shared: &Rc<AppShared>,
    host_id: HostId,
    kind: SurfaceKind,
    placement: Option<SavedPlacement>,
) -> usize {
    let adapter = shared
        .render_bridge
        .get_adapter(&host_id)
        .expect("adapter must exist for host_id");

    let mut surfaces = shared.surfaces.borrow_mut();
    if let Some((idx, surface)) = surfaces
        .iter_mut()
        .enumerate()
        .find(|(_, s)| s.host_id == host_id)
    {
        surface.prepare_recreation(adapter, placement);
        return idx;
    }

    let idx = surfaces.len();
    surfaces.push(SurfaceState {
        host_id,
        kind,
        adapter,
        placement,
        visible: true,
    });
    idx
}

pub fn hide_host(shared: &AppShared, host_id: &HostId) {
    let mut surfaces = shared.surfaces.borrow_mut();
    for (idx, surf) in surfaces
        .iter_mut()
        .enumerate()
        .filter(|(_, s)| &s.host_id == host_id)
    {
        surf.visible = false;
        shared.surface_manager.borrow_mut().retire(host_id);
        let dragging_surface = shared.drag.borrow().as_ref().map(|drag| drag.idx) == Some(idx);
        if dragging_surface {
            shared.drag.borrow_mut().take();
        }
    }
}

pub fn hide_kind(shared: &AppShared, kind: SurfaceKind) {
    let mut surfaces = shared.surfaces.borrow_mut();
    for (idx, surf) in surfaces
        .iter_mut()
        .enumerate()
        .filter(|(_, s)| s.kind == kind)
    {
        surf.visible = false;
        shared.surface_manager.borrow_mut().retire(&surf.host_id);
        let dragging_surface = shared.drag.borrow().as_ref().map(|drag| drag.idx) == Some(idx);
        if dragging_surface {
            shared.drag.borrow_mut().take();
        }
    }
}

pub fn cancel_drag_if_output_missing(
    shared: &AppShared,
    snapshot: &crate::platform::TopologySnapshot,
) -> bool {
    let missing = shared
        .drag
        .borrow()
        .as_ref()
        .is_some_and(|drag| snapshot.get(drag.output_id).is_none());
    if missing {
        shared.drag.borrow_mut().take();
        true
    } else {
        false
    }
}

pub fn process_lifecycle_event(shared: &AppShared, event: PlatformLifecycleEvent) {
    match event {
        PlatformLifecycleEvent::FrameRendered { host_id, .. } => {
            if let Some(idx) = surface_index_for_host(shared, &host_id) {
                if shared.surfaces.borrow()[idx].kind == SurfaceKind::Bar
                    && !shared.first_frame.replace(true)
                {
                    instrumentation::milestone("first-frame");
                    instrumentation::milestone("shell-ready");
                }
            }
        }
        PlatformLifecycleEvent::Closed { host_id, .. } => {
            if let Some(idx) = surface_index_for_host(shared, &host_id) {
                let kind = shared.surfaces.borrow()[idx].kind;
                // S1: Bar close requests application exit
                if kind == SurfaceKind::Bar {
                    shared.exit.set(true);
                }
                // S2: Native surface close cancels active drag targeting that surface
                let dragging_this = shared.drag.borrow().as_ref().map(|drag| drag.idx) == Some(idx);
                if dragging_this {
                    shared.drag.borrow_mut().take();
                }
            }
        }
    }
}

pub fn pump(
    platform: &mut WaylandPlatform,
    shared: &Rc<AppShared>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = platform.dispatch_pending()?;

    if platform.is_topology_dirty() {
        platform.clear_topology_dirty();
        platform.refresh_outputs();
        let has_outputs = !shared.output_manager.borrow().snapshot().is_empty();
        if has_outputs && !shared.outputs_ready.replace(true) {
            instrumentation::milestone("outputs-ready");
        }

        // S2: Invalidate active drag if the target output was removed
        let output_manager = shared.output_manager.borrow();
        let snapshot = output_manager.snapshot();
        cancel_drag_if_output_missing(shared, &snapshot);
        drop(output_manager);

        reconcile(shared);
    }

    // Process generic platform lifecycle events (S1 & S2)
    while let Some(lifecycle_ev) = platform.pop_lifecycle_event() {
        process_lifecycle_event(shared, lifecycle_ev);
    }

    // Process typed pointer input events for drag interactions
    while let Some(ptr_ev) = platform.pop_pointer_event() {
        match ptr_ev {
            PlatformPointerEvent::Press {
                host_id,
                position,
                button: PointerEventButton::Left,
            } => {
                if let Some(idx) = surface_index_for_host(shared, &host_id) {
                    let kind = shared.surfaces.borrow()[idx].kind;
                    if kind == SurfaceKind::Calendar {
                        let mods = platform.modifiers();
                        let handle = position.1 < HANDLE_HEIGHT;
                        if (mods.ctrl && mods.alt) || handle {
                            if let Some(inst) =
                                shared.surface_manager.borrow().get_instance(&host_id)
                            {
                                *shared.drag.borrow_mut() = Some(DragState {
                                    idx,
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
                if let Some(drag) = shared.drag.borrow_mut().take() {
                    if let Some(idx) = surface_index_for_host(shared, &host_id) {
                        if drag.idx == idx {
                            finish_drag(shared, &drag, position);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Drain typed keyboard events if any
    while let Some(_kb_ev) = platform.pop_keyboard_event() {}

    let n = shared.surfaces.borrow().len();
    for i in 0..n {
        let (host_id, _adapter, needs_attach, visible) = {
            let surfaces = shared.surfaces.borrow();
            let s = &surfaces[i];
            let has_native = shared
                .surface_manager
                .borrow()
                .has_native_instance(&s.host_id);
            (
                s.host_id.clone(),
                s.adapter.clone(),
                !has_native && s.visible,
                s.visible,
            )
        };
        if needs_attach {
            try_attach(platform, shared, i);
        }
        if visible {
            platform.render_surface(&host_id);
        }
    }
    platform.flush()?;
    Ok(())
}

fn resolve_ctx(shared: &AppShared) -> PlacementContext {
    PlacementContext {
        spawn_output_id: bar_output_id(shared),
        preferred_name: shared.output_manager.borrow().preferred_output_name(),
        focused_output_id: None,
    }
}

fn bar_output_id(shared: &AppShared) -> Option<OutputId> {
    shared.surfaces.borrow().iter().find_map(|s| {
        if s.kind == SurfaceKind::Bar {
            shared
                .surface_manager
                .borrow()
                .output_id_for_host(&s.host_id)
        } else {
            None
        }
    })
}

fn reconcile(shared: &AppShared) {
    let n = shared.surfaces.borrow().len();
    let ctx = resolve_ctx(shared);
    let output_manager = shared.output_manager.borrow();
    let snapshot = output_manager.snapshot();
    let placement_candidates: Vec<OutputSnapshot> =
        snapshot.placement_capable_outputs().cloned().collect();
    drop(output_manager);

    for i in 0..n {
        let (host_id, adapter, visible, pref) = {
            let surfaces = shared.surfaces.borrow();
            let s = &surfaces[i];
            (
                s.host_id.clone(),
                s.adapter.clone(),
                s.visible,
                s.placement
                    .as_ref()
                    .map(|p| p.output.clone())
                    .unwrap_or_default(),
            )
        };
        if !visible {
            continue;
        }
        let current = shared.surface_manager.borrow().output_id_for_host(&host_id);
        let result = resolve(&pref, &placement_candidates, &ctx);
        let target = result.target_output_id();
        let name = target.and_then(|id| {
            placement_candidates
                .iter()
                .find(|o| o.id == id)
                .and_then(|o| o.name.clone())
        });
        instrumentation::placement_decision(
            match shared.surfaces.borrow()[i].kind {
                SurfaceKind::Bar => "bar",
                SurfaceKind::Calendar => "calendar",
            },
            format!("{:?}", result.reason()).as_str(),
            name.as_deref(),
            result.is_fallback(),
        );
        if target.is_none() {
            shared.surface_manager.borrow_mut().retire(&host_id);
            continue;
        }
        if current != target {
            shared.surface_manager.borrow_mut().retire(&host_id);
            WindowAdapter::request_redraw(&*adapter.window);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachGeometry {
    pub anchor: Anchor,
    pub margins: (i32, i32, i32, i32),
    pub requested_size: (u32, u32),
    pub effective_widget_size: (i32, i32),
    pub offset: (i32, i32),
}

pub fn compute_attach_geometry(
    kind: SurfaceKind,
    placement: Option<&SavedPlacement>,
    preferred: LogicalSize,
    output_logical_size: Option<(i32, i32)>,
) -> AttachGeometry {
    match kind {
        SurfaceKind::Bar => AttachGeometry {
            anchor: Anchor::TopLeft,
            margins: (0, 0, 0, 0),
            requested_size: (0, BAR_HEIGHT),
            effective_widget_size: (0, BAR_HEIGHT as i32),
            offset: (0, 0),
        },
        SurfaceKind::Calendar => {
            let anchor = placement.map(|p| p.anchor).unwrap_or(Anchor::TopRight);
            let raw_offset = placement.map(|p| p.offset).unwrap_or((24, 40));
            let widget_w = preferred.width.max(1.0) as i32;
            let widget_h = preferred.height.max(1.0) as i32;
            let offset = clamp_offset(
                anchor,
                raw_offset,
                (widget_w, widget_h),
                output_logical_size,
                None,
                MIN_VISIBLE,
            );
            let margins = margins_from_offset(anchor, offset);
            let w = widget_w as u32;
            let h = widget_h as u32;
            AttachGeometry {
                anchor,
                margins,
                requested_size: (w, h),
                effective_widget_size: (widget_w, widget_h),
                offset,
            }
        }
    }
}

pub fn surface_spec_for(kind: SurfaceKind) -> SurfaceSpec {
    match kind {
        SurfaceKind::Bar => SurfaceSpec {
            namespace: "owlenshell-bar".into(),
            layer: HostLayer::Top,
            mode: HostMode::Cover,
            size: HostSize::OutputWidth { height: BAR_HEIGHT },
            exclusive_zone: ExclusiveZonePolicy::Fixed(BAR_HEIGHT),
            keyboard_interactivity: KeyboardInteractivityPolicy::None,
        },
        SurfaceKind::Calendar => SurfaceSpec {
            namespace: "owlenshell-calendar".into(),
            layer: HostLayer::Top,
            mode: HostMode::Float,
            size: HostSize::ContentPreferred,
            exclusive_zone: ExclusiveZonePolicy::None,
            keyboard_interactivity: KeyboardInteractivityPolicy::OnDemand,
        },
    }
}

fn try_attach(platform: &mut WaylandPlatform, shared: &AppShared, idx: usize) {
    let (host_id, kind, placement, adapter) = {
        let surfaces = shared.surfaces.borrow();
        let s = &surfaces[idx];
        (
            s.host_id.clone(),
            s.kind,
            s.placement.clone(),
            s.adapter.clone(),
        )
    };
    let preferred = adapter.preferred.get();
    let pref = placement
        .as_ref()
        .map(|p| p.output.clone())
        .unwrap_or_default();
    let ctx = resolve_ctx(shared);
    let output_manager = shared.output_manager.borrow();
    let snapshot = output_manager.snapshot();
    let placement_candidates: Vec<OutputSnapshot> =
        snapshot.placement_capable_outputs().cloned().collect();
    let result = resolve(&pref, &placement_candidates, &ctx);
    let Some(output_id) = result.target_output_id() else {
        return;
    };
    let snap = placement_candidates
        .iter()
        .find(|o| o.id == output_id)
        .cloned();
    let Some(snap) = snap else {
        return;
    };
    let Some(output) = output_manager.get_wl_output(output_id) else {
        return;
    };
    drop(output_manager);

    let spec = surface_spec_for(kind);
    let geo = compute_attach_geometry(kind, placement.as_ref(), preferred, snap.logical_size);

    let params = crate::platform::NativeSurfaceParams {
        host_id,
        spec,
        output_id,
        output_scale: snap.scale,
        anchor: geo.anchor,
        margins: geo.margins,
        requested_size: geo.requested_size,
    };

    platform.attach_surface(params, &output);
    WindowAdapter::request_redraw(&*adapter.window);
}

fn surface_index_for_host(shared: &AppShared, host_id: &HostId) -> Option<usize> {
    shared
        .surfaces
        .borrow()
        .iter()
        .position(|s| &s.host_id == host_id)
}

fn finish_drag(shared: &AppShared, drag: &DragState, pointer: (f64, f64)) {
    let (host_id, widget, margins) = {
        let surfaces = shared.surfaces.borrow();
        let surface = &surfaces[drag.idx];
        let pref = surface.adapter.preferred.get();
        let current_margins = shared
            .surface_manager
            .borrow()
            .get_instance(&surface.host_id)
            .map(|inst| inst.margins)
            .unwrap_or(drag.start_margins);
        (
            surface.host_id.clone(),
            (pref.width.max(1.0) as i32, pref.height.max(1.0) as i32),
            current_margins,
        )
    };
    let output_manager = shared.output_manager.borrow();
    let snapshot = output_manager.snapshot();
    let outputs: Vec<OutputSnapshot> = snapshot.placement_capable_outputs().cloned().collect();
    let commit = PlacementEngine::commit_move(
        pointer,
        drag.grab,
        widget,
        margins,
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

    {
        let mut store = shared.store.borrow_mut();
        store.set(
            crate::domain::ids::HostId::new(CALENDAR_KEY),
            placement.clone(),
        );
        if let Err(err) = store.save() {
            tracing::warn!(%err, "failed to persist placement");
        }
    }
    let recreate = target_output_id != drag.output_id;
    {
        let mut surfaces = shared.surfaces.borrow_mut();
        surfaces[drag.idx].placement = Some(placement.clone());
        if recreate {
            shared.surface_manager.borrow_mut().retire(&host_id);
        } else {
            let new_margins = margins_from_offset(placement.anchor, placement.offset);
            shared.surface_manager.borrow_mut().reconfigure_placement(
                &host_id,
                placement.anchor,
                new_margins,
            );
        }
    }
    tracing::info!(
        output = placement.output.name.as_deref(),
        anchor = ?placement.anchor,
        offset = ?placement.offset,
        recreate,
        "calendar placed"
    );
}
