use crate::domain::ids::HostId;
use crate::instrumentation;
use crate::platform::output_manager::OutputManager;
use crate::platform::render_bridge::RenderBridge;
use crate::platform::surface_manager::{NativeSurfaceParams, SurfaceManager, SurfaceToken};
use calloop::{EventLoop, RegistrationToken};
use calloop_wayland_source::WaylandSource;
use slint::platform::{PointerEventButton, WindowAdapter, WindowEvent};
use slint::LogicalPosition;
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
use std::collections::VecDeque;
use std::rc::Rc;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface};
use wayland_client::{backend::WaylandError, Connection, DispatchError, EventQueue, QueueHandle};

/// Typed keyboard input event emitted across the platform boundary.
#[derive(Debug, Clone)]
pub enum PlatformKeyboardEvent {
    FocusChanged { host_id: Option<HostId> },
    KeyPressed { host_id: HostId, text: String },
    KeyReleased { host_id: HostId, text: String },
    ModifiersChanged { modifiers: Modifiers },
}

/// Typed pointer input event emitted across the platform boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum PlatformPointerEvent {
    Enter {
        host_id: HostId,
        position: (f64, f64),
    },
    Leave {
        host_id: HostId,
    },
    Motion {
        host_id: HostId,
        position: (f64, f64),
    },
    Press {
        host_id: HostId,
        position: (f64, f64),
        button: PointerEventButton,
    },
    Release {
        host_id: HostId,
        position: (f64, f64),
        button: PointerEventButton,
    },
    Axis {
        host_id: HostId,
        position: (f64, f64),
        delta_x: f64,
        delta_y: f64,
    },
}

/// Typed generic surface lifecycle event emitted across the platform boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformLifecycleEvent {
    FrameRendered {
        host_id: HostId,
        token: SurfaceToken,
    },
    Closed {
        host_id: HostId,
        token: SurfaceToken,
    },
}

/// Concrete platform owner of raw Wayland connection, globals, SCTK dispatch, and input translation.
pub struct WaylandPlatform {
    pub(crate) state: WaylandState,
    pub(crate) event_queue: Option<EventQueue<WaylandState>>,
}

/// Internal SCTK callback dispatch state.
pub struct WaylandState {
    pub(crate) conn: Connection,
    pub(crate) qh: QueueHandle<WaylandState>,
    pub(crate) registry: RegistryState,
    pub(crate) compositor: CompositorState,
    pub(crate) layer_shell: LayerShell,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,
    pub(crate) shm: Shm,
    pub(crate) pointer: Option<wl_pointer::WlPointer>,
    pub(crate) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(crate) keyboard_focus: Option<HostId>,
    pub(crate) modifiers: Modifiers,
    pub(crate) output_manager: Rc<RefCell<OutputManager>>,
    pub(crate) surface_manager: Rc<RefCell<SurfaceManager>>,
    pub(crate) render_bridge: Rc<RenderBridge>,
    pub(crate) pending_pointer_events: VecDeque<PlatformPointerEvent>,
    pub(crate) pending_keyboard_events: VecDeque<PlatformKeyboardEvent>,
    pub(crate) pending_lifecycle_events: VecDeque<PlatformLifecycleEvent>,
    pub(crate) topology_dirty: Cell<bool>,
}

impl WaylandPlatform {
    /// Connects to the Wayland environment, binds required globals, and sets up initial dispatch state.
    pub fn connect(
        output_manager: Rc<RefCell<OutputManager>>,
        surface_manager: Rc<RefCell<SurfaceManager>>,
        render_bridge: Rc<RenderBridge>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)?;
        let layer_shell = LayerShell::bind(&globals, &qh)?;
        let shm = Shm::bind(&globals, &qh)?;
        let registry = RegistryState::new(&globals);
        let seat_state = SeatState::new(&globals, &qh);
        let output_state = OutputState::new(&globals, &qh);

        instrumentation::milestone("wayland-bound");

        let state = WaylandState {
            conn,
            qh,
            registry,
            compositor,
            layer_shell,
            seat_state,
            output_state,
            shm,
            pointer: None,
            keyboard: None,
            keyboard_focus: None,
            modifiers: Modifiers::default(),
            output_manager,
            surface_manager,
            render_bridge,
            pending_pointer_events: VecDeque::new(),
            pending_keyboard_events: VecDeque::new(),
            pending_lifecycle_events: VecDeque::new(),
            topology_dirty: Cell::new(true),
        };

        Ok(Self {
            state,
            event_queue: Some(event_queue),
        })
    }

    /// Registers the Wayland event queue into a Calloop event loop via WaylandSource.
    pub fn insert_into_event_loop(
        &mut self,
        event_loop: &EventLoop<WaylandPlatform>,
    ) -> Result<RegistrationToken, Box<dyn std::error::Error>> {
        let event_queue = self
            .event_queue
            .take()
            .ok_or("event queue already inserted into event loop")?;
        let source = WaylandSource::new(self.state.conn.clone(), event_queue);
        let token = event_loop
            .handle()
            .insert_source(source, |_event, queue, platform| {
                queue.dispatch_pending(&mut platform.state)
            })?;
        Ok(token)
    }

    /// Dispatches pending Wayland protocol events without blocking.
    pub fn dispatch_pending(&mut self) -> Result<usize, DispatchError> {
        if let Some(queue) = self.event_queue.as_mut() {
            queue.dispatch_pending(&mut self.state)
        } else {
            // When event_queue is managed by WaylandSource inside Calloop, dispatch occurs via event_loop.dispatch
            Ok(0)
        }
    }

    /// Flushes outgoing Wayland protocol requests.
    pub fn flush(&self) -> Result<(), WaylandError> {
        self.state.conn.flush()
    }

    /// Returns whether any output topology change occurred during dispatch.
    pub fn is_topology_dirty(&self) -> bool {
        self.state.topology_dirty.get()
    }

    /// Acknowledges/clears the topology dirty flag.
    pub fn clear_topology_dirty(&self) {
        self.state.topology_dirty.set(false);
    }

    /// Signals topology dirty explicitly (e.g. after output removal reconciliation).
    pub fn set_topology_dirty(&self) {
        self.state.topology_dirty.set(true);
    }

    /// Returns currently tracked keyboard modifiers.
    pub fn modifiers(&self) -> Modifiers {
        self.state.modifiers
    }

    /// Clears keyboard focus and modifiers.
    pub fn clear_keyboard_focus(&mut self) {
        self.state.keyboard_focus = None;
        self.state.modifiers = Modifiers::default();
    }

    /// Returns currently focused HostId for keyboard input.
    pub fn keyboard_focus(&self) -> Option<&HostId> {
        self.state.keyboard_focus.as_ref()
    }

    /// Pops the next pending platform pointer event.
    pub fn pop_pointer_event(&mut self) -> Option<PlatformPointerEvent> {
        self.state.pending_pointer_events.pop_front()
    }

    /// Pops the next pending platform keyboard event.
    pub fn pop_keyboard_event(&mut self) -> Option<PlatformKeyboardEvent> {
        self.state.pending_keyboard_events.pop_front()
    }

    /// Pops the next pending platform lifecycle event.
    pub fn pop_lifecycle_event(&mut self) -> Option<PlatformLifecycleEvent> {
        self.state.pending_lifecycle_events.pop_front()
    }

    /// Refreshes canonical output inventory from SCTK output state.
    pub fn refresh_outputs(&mut self) {
        let _ = self
            .state
            .output_manager
            .borrow_mut()
            .refresh_sctk(&self.state.output_state);
    }

    /// Attaches or recreates a native surface instance using SurfaceManager.
    pub fn attach_surface(
        &mut self,
        params: NativeSurfaceParams,
        wl_output: &wl_output::WlOutput,
    ) -> SurfaceToken {
        self.state.surface_manager.borrow_mut().attach(
            params,
            wl_output,
            &self.state.compositor,
            &self.state.layer_shell,
            &self.state.qh,
        )
    }

    /// Synchronizes size and draws the surface if configured and needed.
    pub fn render_surface(&mut self, host_id: &HostId) {
        let Some(adapter) = self.state.render_bridge.get_adapter(host_id) else {
            return;
        };
        self.state
            .surface_manager
            .borrow_mut()
            .sync_size(host_id, &adapter);
        self.state.surface_manager.borrow_mut().draw_surface(
            host_id,
            &adapter,
            &self.state.shm,
            &self.state.qh,
        );
    }
}

// ---------------------------------------------------------------------------
// SCTK Callback Delegates
// ---------------------------------------------------------------------------

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if let Some((token, _scale)) = self
            .surface_manager
            .borrow_mut()
            .handle_scale(surface, new_factor)
        {
            if let Some(adapter) = self.render_bridge.get_adapter(&token.host_id) {
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
        if let Some(token) = self.surface_manager.borrow_mut().handle_frame(surface) {
            self.pending_lifecycle_events
                .push_back(PlatformLifecycleEvent::FrameRendered {
                    host_id: token.host_id.clone(),
                    token,
                });
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

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.topology_dirty.set(true);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        self.topology_dirty.set(true);
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let _ = self.output_manager.borrow_mut().remove_wl_output(&output);
        self.topology_dirty.set(true);
    }
}

impl LayerShellHandler for WaylandState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        let token = self.surface_manager.borrow().find_token_by_layer(layer);

        if let Some(token) = token {
            let host_id = token.host_id.clone();
            self.surface_manager.borrow_mut().retire(&host_id);
            if let Some(adapter) = self.render_bridge.get_adapter(&host_id) {
                WindowAdapter::request_redraw(&*adapter.window);
            }
            if self.keyboard_focus.as_ref() == Some(&host_id) {
                self.keyboard_focus = None;
                self.modifiers = Modifiers::default();
                self.pending_keyboard_events
                    .push_back(PlatformKeyboardEvent::FocusChanged { host_id: None });
            }
            self.pending_lifecycle_events
                .push_back(PlatformLifecycleEvent::Closed { host_id, token });
            self.topology_dirty.set(true);
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
            .surface_manager
            .borrow_mut()
            .handle_configure(layer, configure.new_size)
        {
            if let Some(adapter) = self.render_bridge.get_adapter(&token.host_id) {
                WindowAdapter::request_redraw(&*adapter.window);
            }
        }
    }
}

impl SeatHandler for WaylandState {
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
            self.modifiers = Modifiers::default();
            self.keyboard_focus = None;
            self.pending_keyboard_events
                .push_back(PlatformKeyboardEvent::FocusChanged { host_id: None });
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
        self.modifiers = Modifiers::default();
        self.keyboard_focus = None;
        self.pending_keyboard_events
            .push_back(PlatformKeyboardEvent::FocusChanged { host_id: None });
        if let Some(kb) = self.keyboard.take() {
            kb.release();
        }
        if let Some(ptr) = self.pointer.take() {
            ptr.release();
        }
    }
}

impl KeyboardHandler for WaylandState {
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
        let host_id = self
            .surface_manager
            .borrow()
            .find_token_by_surface(surface)
            .map(|t| t.host_id);
        self.keyboard_focus = host_id.clone();
        self.pending_keyboard_events
            .push_back(PlatformKeyboardEvent::FocusChanged { host_id });
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        let leaving_id = self
            .surface_manager
            .borrow()
            .find_token_by_surface(surface)
            .map(|t| t.host_id);
        if self.keyboard_focus == leaving_id {
            self.keyboard_focus = None;
            self.modifiers = Modifiers::default();
            self.pending_keyboard_events
                .push_back(PlatformKeyboardEvent::FocusChanged { host_id: None });
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
            if let Some(host_id) = self.keyboard_focus.as_ref() {
                if let Some(adapter) = self.render_bridge.get_adapter(host_id) {
                    let _ = adapter.window.try_dispatch_event(WindowEvent::KeyPressed {
                        text: text.clone().into(),
                    });
                }
                self.pending_keyboard_events
                    .push_back(PlatformKeyboardEvent::KeyPressed {
                        host_id: host_id.clone(),
                        text,
                    });
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
            if let Some(host_id) = self.keyboard_focus.as_ref() {
                if let Some(adapter) = self.render_bridge.get_adapter(host_id) {
                    let _ = adapter.window.try_dispatch_event(WindowEvent::KeyReleased {
                        text: text.clone().into(),
                    });
                }
                self.pending_keyboard_events
                    .push_back(PlatformKeyboardEvent::KeyReleased {
                        host_id: host_id.clone(),
                        text,
                    });
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
        self.modifiers = modifiers;
        self.pending_keyboard_events
            .push_back(PlatformKeyboardEvent::ModifiersChanged { modifiers });
    }
}

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(token) = self
                .surface_manager
                .borrow()
                .find_token_by_surface(&event.surface)
            else {
                continue;
            };
            let host_id = token.host_id;
            let adapter = self.render_bridge.get_adapter(&host_id);
            let pos = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

            match event.kind {
                PointerEventKind::Enter { .. } => {
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerMoved { position: pos });
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Enter {
                            host_id,
                            position: event.position,
                        });
                }
                PointerEventKind::Motion { .. } => {
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerMoved { position: pos });
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Motion {
                            host_id,
                            position: event.position,
                        });
                }
                PointerEventKind::Leave { .. } => {
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerExited);
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Leave { host_id });
                }
                PointerEventKind::Press { button, .. } => {
                    let btn = if button == 0x110 {
                        PointerEventButton::Left
                    } else if button == 0x111 {
                        PointerEventButton::Right
                    } else {
                        PointerEventButton::Middle
                    };
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerPressed {
                                position: pos,
                                button: btn,
                            });
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Press {
                            host_id,
                            position: event.position,
                            button: btn,
                        });
                }
                PointerEventKind::Release { button, .. } => {
                    let btn = if button == 0x110 {
                        PointerEventButton::Left
                    } else if button == 0x111 {
                        PointerEventButton::Right
                    } else {
                        PointerEventButton::Middle
                    };
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerReleased {
                                position: pos,
                                button: btn,
                            });
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Release {
                            host_id,
                            position: event.position,
                            button: btn,
                        });
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    if let Some(adapter) = adapter.as_ref() {
                        let _ = adapter
                            .window
                            .try_dispatch_event(WindowEvent::PointerScrolled {
                                position: pos,
                                delta_x: horizontal.absolute as f32,
                                delta_y: vertical.absolute as f32,
                            });
                    }
                    self.pending_pointer_events
                        .push_back(PlatformPointerEvent::Axis {
                            host_id,
                            position: event.position,
                            delta_x: horizontal.absolute,
                            delta_y: vertical.absolute,
                        });
                }
            }
        }
    }
}

impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(WaylandState);

impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_dispatch2!(WaylandState);
