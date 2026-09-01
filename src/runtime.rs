pub mod placement_engine;

use crate::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use crate::domain::ids::{HostId, OutputId};
use crate::instrumentation;
pub use crate::model::SurfaceKind;
use crate::model::{
    Anchor, OutputSnapshot, SavedPlacement, BAR_HEIGHT, BTN_LEFT, CALENDAR_KEY, HANDLE_HEIGHT,
};
use crate::persistence::Placements;
use crate::placement::{
    clamp_offset, margins_from_offset, resolve, PlacementContext, PlacementEngine, MIN_VISIBLE,
};
use crate::platform::{
    dispatch_pointer, OutputManager, OwlenWindowAdapter, RenderBridge, SurfaceManager, SurfaceSpec,
};
use slint::platform::{PointerEventButton, WindowAdapter, WindowEvent};
use slint::{LogicalPosition, LogicalSize};
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
use smithay_client_toolkit::delegate_registry;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::registry_handlers;
use smithay_client_toolkit::seat::keyboard::{
    KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers,
};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::wlr_layer::{
    LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface};
use wayland_client::{backend::WaylandError, Connection, QueueHandle};

pub struct AppShared {
    pub conn: Connection,
    pub qh: QueueHandle<AppState>,
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub surfaces: RefCell<Vec<SurfaceState>>,
    pub surface_manager: RefCell<SurfaceManager>,
    pub render_bridge: Rc<RenderBridge>,
    pub output_manager: RefCell<OutputManager>,
    pub store: RefCell<Placements>,
    pub modifiers: Cell<Modifiers>,
    pub keyboard_focus: Cell<Option<usize>>,
    pub drag: RefCell<Option<DragState>>,
    pub topology_dirty: Cell<bool>,
    pub exit: Cell<bool>,
    pub first_frame: Cell<bool>,
    pub outputs_ready: Cell<bool>,
}

pub struct AppState {
    pub registry: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub shared: Rc<AppShared>,
}

pub struct SurfaceState {
    pub host_id: HostId,
    pub kind: SurfaceKind,
    pub adapter: Rc<OwlenWindowAdapter>,
    pub placement: Option<SavedPlacement>,
    pub visible: bool,
}

pub struct DragState {
    idx: usize,
    grab: (f64, f64),
    start_margins: (i32, i32, i32, i32),
    anchor: Anchor,
    output_id: OutputId,
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
        if shared.keyboard_focus.get() == Some(idx) {
            shared.keyboard_focus.set(None);
            shared.modifiers.set(Modifiers::default());
        }
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
        if shared.keyboard_focus.get() == Some(idx) {
            shared.keyboard_focus.set(None);
            shared.modifiers.set(Modifiers::default());
        }
        let dragging_surface = shared.drag.borrow().as_ref().map(|drag| drag.idx) == Some(idx);
        if dragging_surface {
            shared.drag.borrow_mut().take();
        }
    }
}

pub fn pump(state: &mut AppState) -> Result<(), WaylandError> {
    if state.shared.topology_dirty.replace(false) {
        refresh_outputs(state);
        reconcile(&state.shared);
    }
    let n = state.shared.surfaces.borrow().len();
    for i in 0..n {
        let (host_id, adapter, needs_attach, visible) = {
            let surfaces = state.shared.surfaces.borrow();
            let s = &surfaces[i];
            let has_native = state
                .shared
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
            try_attach(&state.shared, i);
        }
        if visible {
            state
                .shared
                .surface_manager
                .borrow_mut()
                .sync_size(&host_id, &adapter);
            state.shared.surface_manager.borrow_mut().draw_surface(
                &host_id,
                &adapter,
                &state.shm,
                &state.shared.qh,
            );
        }
    }
    state.shared.conn.flush()
}

fn refresh_outputs(state: &mut AppState) {
    let _ = state
        .shared
        .output_manager
        .borrow_mut()
        .refresh_sctk(&state.output_state);
    let has_outputs = !state.shared.output_manager.borrow().snapshot().is_empty();
    if has_outputs && !state.shared.outputs_ready.replace(true) {
        instrumentation::milestone("outputs-ready");
    }
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

fn try_attach(shared: &AppShared, idx: usize) {
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

    shared.surface_manager.borrow_mut().attach(
        params,
        &output,
        &shared.compositor,
        &shared.layer_shell,
        &shared.qh,
    );
    WindowAdapter::request_redraw(&*adapter.window);
}

fn surface_index_for_host(shared: &AppShared, host_id: &HostId) -> Option<usize> {
    shared
        .surfaces
        .borrow()
        .iter()
        .position(|s| &s.host_id == host_id)
}

fn surface_index_for(shared: &AppShared, surface: &wl_surface::WlSurface) -> Option<usize> {
    let token = shared
        .surface_manager
        .borrow()
        .find_token_by_surface(surface)?;
    surface_index_for_host(shared, &token.host_id)
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if let Some((token, _scale)) = self
            .shared
            .surface_manager
            .borrow_mut()
            .handle_scale(surface, new_factor)
        {
            if let Some(idx) = surface_index_for_host(&self.shared, &token.host_id) {
                let adapter = self.shared.surfaces.borrow()[idx].adapter.clone();
                WindowAdapter::request_redraw(&*adapter.window);
            }
        }
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        if let Some(token) = self
            .shared
            .surface_manager
            .borrow_mut()
            .handle_frame(surface)
        {
            if let Some(idx) = surface_index_for_host(&self.shared, &token.host_id) {
                if self.shared.surfaces.borrow()[idx].kind == SurfaceKind::Bar
                    && !self.shared.first_frame.replace(true)
                {
                    instrumentation::milestone("first-frame");
                    instrumentation::milestone("shell-ready");
                }
            }
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.shared.topology_dirty.set(true);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.shared.topology_dirty.set(true);
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let removed = self
            .shared
            .output_manager
            .borrow_mut()
            .remove_wl_output(&output);
        let removed_id = removed.as_ref().map(|(id, _, _)| *id);

        let drag_targets_removed_output = self
            .shared
            .drag
            .borrow()
            .as_ref()
            .is_some_and(|drag| Some(drag.output_id) == removed_id);
        if drag_targets_removed_output {
            self.shared.drag.borrow_mut().take();
        }

        reconcile(&self.shared);
        let surface_count = self.shared.surfaces.borrow().len();
        for idx in 0..surface_count {
            let (host_id, visible) = {
                let surfaces = self.shared.surfaces.borrow();
                (surfaces[idx].host_id.clone(), surfaces[idx].visible)
            };
            let needs_attach = visible
                && !self
                    .shared
                    .surface_manager
                    .borrow()
                    .has_native_instance(&host_id);
            if needs_attach {
                try_attach(&self.shared, idx);
            }
        }
        self.shared.topology_dirty.set(true);
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        if let Some(token) = self
            .shared
            .surface_manager
            .borrow()
            .find_token_by_layer(layer)
        {
            let host_id = token.host_id.clone();
            self.shared.surface_manager.borrow_mut().retire(&host_id);
            if let Some(idx) = surface_index_for_host(&self.shared, &host_id) {
                let (kind, adapter) = {
                    let surfaces = self.shared.surfaces.borrow();
                    (surfaces[idx].kind, surfaces[idx].adapter.clone())
                };
                if kind == SurfaceKind::Bar {
                    self.shared.exit.set(true);
                }
                WindowAdapter::request_redraw(&*adapter.window);
                if self.shared.keyboard_focus.get() == Some(idx) {
                    self.shared.keyboard_focus.set(None);
                    self.shared.modifiers.set(Modifiers::default());
                }
                if self.shared.drag.borrow().as_ref().map(|drag| drag.idx) == Some(idx) {
                    self.shared.drag.borrow_mut().take();
                }
                self.shared.topology_dirty.set(true);
            }
        }
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(token) = self
            .shared
            .surface_manager
            .borrow_mut()
            .handle_configure(layer, configure.new_size)
        {
            if let Some(idx) = surface_index_for_host(&self.shared, &token.host_id) {
                let adapter = self.shared.surfaces.borrow()[idx].adapter.clone();
                WindowAdapter::request_redraw(&*adapter.window);
            }
        }
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            if let Ok(kb) = self.seat_state.get_keyboard(qh, &seat, None) {
                self.keyboard = Some(kb);
            }
        }
        if capability == Capability::Pointer && self.pointer.is_none() {
            if let Ok(ptr) = self.seat_state.get_pointer(qh, &seat) {
                self.pointer = Some(ptr);
            }
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard {
            self.shared.modifiers.set(Modifiers::default());
            self.shared.keyboard_focus.set(None);
            if let Some(kb) = self.keyboard.take() {
                kb.release();
            }
        }
        if capability == Capability::Pointer {
            if let Some(ptr) = self.pointer.take() {
                ptr.release();
            }
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
        self.shared.modifiers.set(Modifiers::default());
        self.shared.keyboard_focus.set(None);
        if let Some(kb) = self.keyboard.take() {
            kb.release();
        }
        if let Some(ptr) = self.pointer.take() {
            ptr.release();
        }
    }
}

impl KeyboardHandler for AppState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        self.shared
            .keyboard_focus
            .set(surface_index_for(&self.shared, surface));
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        let leaving = surface_index_for(&self.shared, surface);
        if self.shared.keyboard_focus.get() == leaving {
            self.shared.keyboard_focus.set(None);
            self.shared.modifiers.set(Modifiers::default());
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if let Some(text) = event.utf8 {
            if let Some(idx) = self.shared.keyboard_focus.get() {
                let adapter = self.shared.surfaces.borrow()[idx].adapter.clone();
                dispatch_pointer(
                    &adapter,
                    WindowEvent::KeyPressed {
                        text: text.as_str().into(),
                    },
                );
            }
        }
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if let Some(text) = event.utf8 {
            if let Some(idx) = self.shared.keyboard_focus.get() {
                let adapter = self.shared.surfaces.borrow()[idx].adapter.clone();
                dispatch_pointer(
                    &adapter,
                    WindowEvent::KeyReleased {
                        text: text.as_str().into(),
                    },
                );
            }
        }
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
        self.shared.modifiers.set(modifiers);
    }
}

impl PointerHandler for AppState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(idx) = surface_index_for(&self.shared, &event.surface) else {
                continue;
            };
            let (host_id, adapter, kind) = {
                let surfaces = self.shared.surfaces.borrow();
                (
                    surfaces[idx].host_id.clone(),
                    surfaces[idx].adapter.clone(),
                    surfaces[idx].kind,
                )
            };
            let pos = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    dispatch_pointer(&adapter, WindowEvent::PointerMoved { position: pos });
                }
                PointerEventKind::Leave { .. } => {
                    dispatch_pointer(&adapter, WindowEvent::PointerExited);
                }
                PointerEventKind::Press { button, .. } => {
                    if button == BTN_LEFT && kind == SurfaceKind::Calendar {
                        let mods = self.shared.modifiers.get();
                        let handle = event.position.1 < HANDLE_HEIGHT;
                        if (mods.ctrl && mods.alt) || handle {
                            if let Some(inst) =
                                self.shared.surface_manager.borrow().get_instance(&host_id)
                            {
                                *self.shared.drag.borrow_mut() = Some(DragState {
                                    idx,
                                    grab: event.position,
                                    start_margins: inst.margins,
                                    anchor: inst.anchor,
                                    output_id: inst.output_id,
                                });
                            }
                        }
                    }
                    dispatch_pointer(
                        &adapter,
                        WindowEvent::PointerPressed {
                            position: pos,
                            button: PointerEventButton::Left,
                        },
                    );
                }
                PointerEventKind::Release { button, .. } => {
                    if button == BTN_LEFT {
                        if let Some(drag) = self.shared.drag.borrow_mut().take() {
                            if drag.idx == idx {
                                finish_drag(&self.shared, &drag, event.position);
                            }
                        }
                    }
                    dispatch_pointer(
                        &adapter,
                        WindowEvent::PointerReleased {
                            position: pos,
                            button: PointerEventButton::Left,
                        },
                    );
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    dispatch_pointer(
                        &adapter,
                        WindowEvent::PointerScrolled {
                            position: pos,
                            delta_x: horizontal.absolute as f32,
                            delta_y: vertical.absolute as f32,
                        },
                    );
                }
            }
        }
    }
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

impl ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_dispatch2!(AppState);
