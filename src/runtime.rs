pub mod placement_engine;

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
use crate::platform::{dispatch_pointer, OutputManager, OwlenWindowAdapter, RenderBridge};
use slint::platform::software_renderer::{PremultipliedRgbaColor, TargetPixel};
use slint::platform::{PointerEventButton, WindowAdapter, WindowEvent};
use slint::{LogicalPosition, LogicalSize, PhysicalSize};
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState, FrameCallbackData};
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
    Anchor as LayerAnchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler,
    LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface};
use wayland_client::{backend::WaylandError, Connection, QueueHandle};

pub struct AppShared {
    pub conn: Connection,
    pub qh: QueueHandle<AppState>,
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub surfaces: RefCell<Vec<SurfaceState>>,
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
    pub scale: i32,
    pub configured: Option<(u32, u32)>,
    pub frame_pending: bool,
    pub render_size: Option<(u32, u32)>,
    pub dispatched_scale: i32,
    pub wl: Option<LiveSurface>,
}

pub struct LiveSurface {
    surface: wl_surface::WlSurface,
    layer: LayerSurface,
    output_id: OutputId,
    anchor: Anchor,
    margins: (i32, i32, i32, i32),
    requested_size: (u32, u32),
    pool: Option<SlotPool>,
    buffer: Option<Buffer>,
}

pub struct DragState {
    idx: usize,
    grab: (f64, f64),
    start_margins: (i32, i32, i32, i32),
    anchor: Anchor,
    output_id: OutputId,
}

impl SurfaceState {
    /// Retires active native surface resources while preserving logical host identity and visibility.
    pub fn retire_native_surface(&mut self) {
        self.wl = None;
        self.configured = None;
        self.render_size = None;
        self.frame_pending = false;
        WindowAdapter::request_redraw(&*self.adapter.window);
    }

    /// Prepares an existing surface state for reattachment or recreation with updated placement.
    pub fn prepare_recreation(
        &mut self,
        adapter: Rc<OwlenWindowAdapter>,
        placement: Option<SavedPlacement>,
    ) {
        self.adapter = adapter;
        self.placement = placement;
        self.visible = true;
        self.retire_native_surface();
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
        scale: 1,
        configured: None,
        frame_pending: false,
        render_size: None,
        dispatched_scale: 0,
        wl: None,
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
        surf.wl = None;
        surf.configured = None;
        surf.render_size = None;
        surf.frame_pending = false;
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
        surf.wl = None;
        surf.configured = None;
        surf.render_size = None;
        surf.frame_pending = false;
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
        let (needs_attach, visible) = {
            let surfaces = state.shared.surfaces.borrow();
            let s = &surfaces[i];
            (s.wl.is_none() && s.visible, s.visible)
        };
        if needs_attach {
            try_attach(&state.shared, i);
        }
        if visible {
            sync_size(&state.shared, i);
            draw_surface(&state.shared, &state.shm, i);
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
            s.wl.as_ref().map(|l| l.output_id)
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
        let (visible, current, pref) = {
            let surfaces = shared.surfaces.borrow();
            let s = &surfaces[i];
            (
                s.visible,
                s.wl.as_ref().map(|l| l.output_id),
                s.placement
                    .as_ref()
                    .map(|p| p.output.clone())
                    .unwrap_or_default(),
            )
        };
        if !visible {
            continue;
        }
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
            let mut surfaces = shared.surfaces.borrow_mut();
            surfaces[i].retire_native_surface();
            continue;
        }
        if current != target {
            let mut surfaces = shared.surfaces.borrow_mut();
            surfaces[i].retire_native_surface();
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

fn try_attach(shared: &AppShared, idx: usize) {
    let (kind, placement, preferred) = {
        let surfaces = shared.surfaces.borrow();
        let s = &surfaces[idx];
        (s.kind, s.placement.clone(), s.adapter.preferred.get())
    };
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

    let wl_surface = shared.compositor.create_surface(&shared.qh);
    let (ns, kb) = match kind {
        SurfaceKind::Bar => ("owlenshell-bar", KeyboardInteractivity::None),
        SurfaceKind::Calendar => ("owlenshell-calendar", KeyboardInteractivity::OnDemand),
    };
    let layer = shared.layer_shell.create_layer_surface(
        &shared.qh,
        wl_surface,
        Layer::Top,
        Some(ns),
        Some(&output),
    );

    let geo = compute_attach_geometry(kind, placement.as_ref(), preferred, snap.logical_size);

    match kind {
        SurfaceKind::Bar => {
            layer.set_anchor(LayerAnchor::TOP | LayerAnchor::LEFT | LayerAnchor::RIGHT);
            layer.set_size(geo.requested_size.0, geo.requested_size.1);
            layer.set_exclusive_zone(BAR_HEIGHT as i32);
        }
        SurfaceKind::Calendar => {
            layer.set_anchor(geo.anchor.as_layer_anchor());
            layer.set_size(geo.requested_size.0, geo.requested_size.1);
            layer.set_exclusive_zone(0);
            layer.set_margin(geo.margins.0, geo.margins.1, geo.margins.2, geo.margins.3);
        }
    }
    layer.set_keyboard_interactivity(kb);
    layer.commit();

    let surface = layer.wl_surface().clone();
    let output_scale = snap.scale.max(1);
    surface.set_buffer_scale(output_scale);

    let mut surfaces = shared.surfaces.borrow_mut();
    surfaces[idx].scale = output_scale;
    surfaces[idx].wl = Some(LiveSurface {
        surface,
        layer,
        output_id,
        anchor: geo.anchor,
        margins: geo.margins,
        requested_size: geo.requested_size,
        pool: None,
        buffer: None,
    });
    surfaces[idx].configured = None;
    surfaces[idx].render_size = None;
    surfaces[idx].frame_pending = false;
    WindowAdapter::request_redraw(&*surfaces[idx].adapter.window);
}

fn sync_size(shared: &AppShared, idx: usize) {
    let mut surfaces = shared.surfaces.borrow_mut();
    let surf = &mut surfaces[idx];
    let Some(live) = surf.wl.as_mut() else {
        return;
    };
    let preferred = surf.adapter.preferred.get();
    let scale_i32 = surf.scale.max(1);
    let scale = scale_i32 as f32;
    if surf.dispatched_scale != scale_i32 {
        let _ = surf
            .adapter
            .window
            .try_dispatch_event(WindowEvent::ScaleFactorChanged {
                scale_factor: scale,
            });
        surf.dispatched_scale = scale_i32;
    }
    let target_size = match surf.kind {
        SurfaceKind::Bar => {
            let (cw, ch) = surf.configured.unwrap_or((0, BAR_HEIGHT));
            let w = if cw == 0 { 1 } else { cw };
            let h = if ch == 0 { BAR_HEIGHT } else { ch };
            ((w as f32 * scale) as u32, (h as f32 * scale) as u32)
        }
        SurfaceKind::Calendar => {
            let lw = preferred.width.max(1.0) as u32;
            let lh = preferred.height.max(1.0) as u32;
            let (cw, ch) = surf.configured.unwrap_or((lw, lh));
            let w = if cw == 0 { lw } else { cw };
            let h = if ch == 0 { lh } else { ch };
            if live.requested_size != (lw, lh) {
                live.layer.set_size(lw, lh);
                live.layer.commit();
                live.requested_size = (lw, lh);
            }
            ((w as f32 * scale) as u32, (h as f32 * scale) as u32)
        }
    };
    if surf.render_size != Some(target_size) {
        surf.adapter
            .window
            .set_size(PhysicalSize::new(target_size.0, target_size.1));
        surf.render_size = Some(target_size);
    }
}

fn draw_surface(shared: &AppShared, shm: &Shm, idx: usize) {
    let mut surfaces = shared.surfaces.borrow_mut();
    let surf = &mut surfaces[idx];
    if surf.configured.is_none() || surf.frame_pending {
        return;
    }
    let Some(live) = surf.wl.as_mut() else {
        return;
    };
    let scale = surf.scale.max(1);
    let preferred = surf.adapter.preferred.get();
    let (cw, ch) = surf.configured.unwrap();
    let (lw, lh) = match surf.kind {
        SurfaceKind::Bar => (
            if cw == 0 { 1 } else { cw },
            if ch == 0 { BAR_HEIGHT } else { ch },
        ),
        SurfaceKind::Calendar => {
            let pw = preferred.width.max(1.0) as u32;
            let ph = preferred.height.max(1.0) as u32;
            (if cw == 0 { pw } else { cw }, if ch == 0 { ph } else { ch })
        }
    };
    let w = (lw as i32) * scale;
    let h = (lh as i32) * scale;
    if w <= 0 || h <= 0 {
        return;
    }
    let mut rendered = None;
    let redrawn = surf.adapter.window.draw_if_needed(|renderer| {
        let mut tmp = vec![PremultipliedRgbaColor::background(); (w * h) as usize];
        renderer.render(&mut tmp, w as usize);
        rendered = Some(tmp);
    });
    if !redrawn {
        return;
    }
    let Some(tmp) = rendered else {
        return;
    };

    let bytes = (w * h * 4) as usize;
    if live.pool.is_none() {
        live.pool = SlotPool::new(bytes.max(4096), shm).ok();
    }
    let Some(pool) = live.pool.as_mut() else {
        WindowAdapter::request_redraw(&*surf.adapter.window);
        return;
    };
    let (buffer, canvas) = match pool.create_buffer(w, h, w * 4, wl_shm::Format::Argb8888) {
        Ok(pair) => pair,
        Err(_) => {
            let _ = pool.resize(bytes * 2);
            let Ok(pair) = pool.create_buffer(w, h, w * 4, wl_shm::Format::Argb8888) else {
                WindowAdapter::request_redraw(&*surf.adapter.window);
                return;
            };
            pair
        }
    };

    for (dst, src) in canvas.as_chunks_mut::<4>().0.iter_mut().zip(tmp.iter()) {
        dst[0] = src.blue;
        dst[1] = src.green;
        dst[2] = src.red;
        dst[3] = src.alpha;
    }

    if buffer.attach_to(&live.surface).is_err() {
        WindowAdapter::request_redraw(&*surf.adapter.window);
        return;
    }
    live.surface.damage_buffer(0, 0, w, h);
    live.surface
        .frame(&shared.qh, FrameCallbackData(live.surface.clone()));
    live.layer.commit();
    live.buffer = Some(buffer);
    surf.frame_pending = true;
}

fn surface_index_for(shared: &AppShared, surface: &wl_surface::WlSurface) -> Option<usize> {
    shared.surfaces.borrow().iter().position(|s| {
        s.wl.as_ref()
            .map(|l| &l.surface == surface)
            .unwrap_or(false)
    })
}

fn layer_index_for(shared: &AppShared, layer: &LayerSurface) -> Option<usize> {
    shared
        .surfaces
        .borrow()
        .iter()
        .position(|s| s.wl.as_ref().map(|l| &l.layer == layer).unwrap_or(false))
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if let Some(idx) = surface_index_for(&self.shared, surface) {
            let mut surfaces = self.shared.surfaces.borrow_mut();
            surfaces[idx].scale = new_factor.max(1);
            if let Some(live) = surfaces[idx].wl.as_ref() {
                live.surface.set_buffer_scale(new_factor.max(1));
            }
            WindowAdapter::request_redraw(&*surfaces[idx].adapter.window);
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
        if let Some(idx) = surface_index_for(&self.shared, surface) {
            let mut surfaces = self.shared.surfaces.borrow_mut();
            surfaces[idx].frame_pending = false;
            if surfaces[idx].kind == SurfaceKind::Bar && !self.shared.first_frame.replace(true) {
                instrumentation::milestone("first-frame");
                instrumentation::milestone("shell-ready");
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
            let needs_attach = {
                let surfaces = self.shared.surfaces.borrow();
                surfaces[idx].visible && surfaces[idx].wl.is_none()
            };
            if needs_attach {
                try_attach(&self.shared, idx);
            }
        }
        self.shared.topology_dirty.set(true);
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        if let Some(idx) = layer_index_for(&self.shared, layer) {
            let mut surfaces = self.shared.surfaces.borrow_mut();
            if surfaces[idx].kind == SurfaceKind::Bar {
                self.shared.exit.set(true);
            }
            surfaces[idx].retire_native_surface();
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

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(idx) = layer_index_for(&self.shared, layer) {
            let mut surfaces = self.shared.surfaces.borrow_mut();
            surfaces[idx].configured = Some(configure.new_size);
            WindowAdapter::request_redraw(&*surfaces[idx].adapter.window);
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
            let adapter = self.shared.surfaces.borrow()[idx].adapter.clone();
            let kind = self.shared.surfaces.borrow()[idx].kind;
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
                            let (margins, anchor, output_id) = {
                                let surfaces = self.shared.surfaces.borrow();
                                let live = surfaces[idx].wl.as_ref().unwrap();
                                (live.margins, live.anchor, live.output_id)
                            };
                            *self.shared.drag.borrow_mut() = Some(DragState {
                                idx,
                                grab: event.position,
                                start_margins: margins,
                                anchor,
                                output_id,
                            });
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
    let (widget, margins) = {
        let surfaces = shared.surfaces.borrow();
        let surface = &surfaces[drag.idx];
        let pref = surface.adapter.preferred.get();
        (
            (pref.width.max(1.0) as i32, pref.height.max(1.0) as i32),
            surface
                .wl
                .as_ref()
                .map(|live| live.margins)
                .unwrap_or(drag.start_margins),
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
            surfaces[drag.idx].retire_native_surface();
        } else if let Some(live) = surfaces[drag.idx].wl.as_mut() {
            live.anchor = placement.anchor;
            live.margins = margins_from_offset(placement.anchor, placement.offset);
            live.layer.set_anchor(placement.anchor.as_layer_anchor());
            live.layer.set_margin(
                live.margins.0,
                live.margins.1,
                live.margins.2,
                live.margins.3,
            );
            live.layer.commit();
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
