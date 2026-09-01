use crate::domain::ids::HostId;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent, WindowProperties};
use slint::{LogicalSize, PhysicalSize, PlatformError, Window, WindowSize};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// Monotonic nonce uniquely identifying a pending window creation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreationNonce(u64);

impl CreationNonce {
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Token representing an in-flight window creation scoped to a specific HostId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWindowCreation {
    pub host_id: HostId,
    pub nonce: CreationNonce,
}

/// Errors originating from RenderBridge operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RenderBridgeError {
    #[error("nested window creation is not allowed")]
    NestedCreation,
    #[error("host '{0}' already has an active window adapter")]
    DuplicateHost(HostId),
    #[error("constructor failed: {0}")]
    ConstructorFailed(String),
    #[error("constructor did not create an adapter for host '{0}'")]
    TokenNotConsumed(HostId),
    #[error("create_window_adapter called without pending HostId creation scope")]
    NoPendingCreation,
    #[error("duplicate adapter creation callback for host '{0}'")]
    DuplicateCallback(HostId),
}

/// Slint WindowAdapter implementation keyed by immutable HostId.
pub struct OwlenWindowAdapter {
    pub host_id: HostId,
    pub window: Rc<MinimalSoftwareWindow>,
    pub preferred: Cell<LogicalSize>,
    pub visible: Cell<bool>,
}

impl OwlenWindowAdapter {
    pub fn new(host_id: HostId) -> Rc<Self> {
        Rc::new(Self {
            host_id,
            window: MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer),
            preferred: Cell::new(LogicalSize::new(32.0, 32.0)),
            visible: Cell::new(true),
        })
    }

    pub fn host_id(&self) -> &HostId {
        &self.host_id
    }
}

impl std::fmt::Debug for OwlenWindowAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwlenWindowAdapter")
            .field("host_id", &self.host_id)
            .field("visible", &self.visible.get())
            .field("preferred", &self.preferred.get())
            .finish()
    }
}

impl WindowAdapter for OwlenWindowAdapter {
    fn window(&self) -> &Window {
        self.window.window()
    }

    fn size(&self) -> PhysicalSize {
        self.window.size()
    }

    fn renderer(&self) -> &dyn slint::platform::Renderer {
        self.window.renderer()
    }

    fn request_redraw(&self) {
        WindowAdapter::request_redraw(&*self.window);
    }

    fn set_visible(&self, visible: bool) -> Result<(), PlatformError> {
        self.visible.set(visible);
        Ok(())
    }

    fn set_size(&self, size: WindowSize) {
        self.window.set_size(size);
    }

    fn update_window_properties(&self, properties: WindowProperties<'_>) {
        self.preferred
            .set(properties.layout_constraints().preferred);
    }
}

/// Dispatches a pointer event to the Slint window.
pub fn dispatch_pointer(adapter: &OwlenWindowAdapter, event: WindowEvent) {
    let _ = adapter.window.try_dispatch_event(event);
}

/// Dispatches a keyboard event to the Slint window.
pub fn dispatch_keyboard(adapter: &OwlenWindowAdapter, event: WindowEvent) {
    let _ = adapter.window.try_dispatch_event(event);
}

#[derive(Debug, Clone)]
struct RegisteredAdapter {
    adapter: Rc<OwlenWindowAdapter>,
    nonce: CreationNonce,
}

/// Coordinates Slint platform integration, adapter registration, and HostId creation scopes.
pub struct RenderBridge {
    pending_creation: RefCell<Option<PendingWindowCreation>>,
    adapters: RefCell<HashMap<HostId, RegisteredAdapter>>,
    next_nonce: Cell<u64>,
}

impl RenderBridge {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            pending_creation: RefCell::new(None),
            adapters: RefCell::new(HashMap::new()),
            next_nonce: Cell::new(1),
        })
    }

    /// Executes the component constructor within a scoped pending `HostId` registration.
    pub fn with_host_creation<T, E, F>(
        &self,
        host_id: HostId,
        constructor: F,
    ) -> Result<T, RenderBridgeError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        if self.pending_creation.borrow().is_some() {
            return Err(RenderBridgeError::NestedCreation);
        }
        if self.adapters.borrow().contains_key(&host_id) {
            return Err(RenderBridgeError::DuplicateHost(host_id));
        }

        let nonce = CreationNonce(self.next_nonce.get());
        self.next_nonce.set(self.next_nonce.get() + 1);

        *self.pending_creation.borrow_mut() = Some(PendingWindowCreation {
            host_id: host_id.clone(),
            nonce,
        });

        struct PendingGuard<'a> {
            bridge: &'a RenderBridge,
            host_id: HostId,
            nonce: CreationNonce,
            completed: bool,
        }

        impl<'a> Drop for PendingGuard<'a> {
            fn drop(&mut self) {
                if !self.completed {
                    let should_clear = self
                        .bridge
                        .pending_creation
                        .borrow()
                        .as_ref()
                        .map(|p| p.nonce == self.nonce)
                        .unwrap_or(false);
                    if should_clear {
                        *self.bridge.pending_creation.borrow_mut() = None;
                    }
                    let mut adapters = self.bridge.adapters.borrow_mut();
                    if let Some(entry) = adapters.get(&self.host_id) {
                        if entry.nonce == self.nonce {
                            adapters.remove(&self.host_id);
                        }
                    }
                }
            }
        }

        let mut guard = PendingGuard {
            bridge: self,
            host_id: host_id.clone(),
            nonce,
            completed: false,
        };

        let result =
            constructor().map_err(|e| RenderBridgeError::ConstructorFailed(e.to_string()))?;

        if self.pending_creation.borrow().is_some() {
            return Err(RenderBridgeError::TokenNotConsumed(host_id));
        }

        if let Some(entry) = self.adapters.borrow().get(&host_id) {
            if entry.nonce != nonce {
                return Err(RenderBridgeError::TokenNotConsumed(host_id));
            }
        } else {
            return Err(RenderBridgeError::TokenNotConsumed(host_id));
        }

        guard.completed = true;
        Ok(result)
    }

    /// Invoked by `Platform::create_window_adapter` to consume the pending creation token.
    pub fn create_window_adapter(&self) -> Result<Rc<OwlenWindowAdapter>, PlatformError> {
        let pending = self.pending_creation.borrow_mut().take().ok_or_else(|| {
            PlatformError::Other(
                "create_window_adapter called without pending HostId creation scope".into(),
            )
        })?;

        let host_id = pending.host_id;
        let nonce = pending.nonce;
        if self.adapters.borrow().contains_key(&host_id) {
            return Err(PlatformError::Other(format!(
                "duplicate adapter creation for host '{host_id}'"
            )));
        }

        let adapter = OwlenWindowAdapter::new(host_id.clone());
        self.adapters.borrow_mut().insert(
            host_id,
            RegisteredAdapter {
                adapter: adapter.clone(),
                nonce,
            },
        );
        Ok(adapter)
    }

    pub fn get_adapter(&self, host_id: &HostId) -> Option<Rc<OwlenWindowAdapter>> {
        self.adapters
            .borrow()
            .get(host_id)
            .map(|entry| entry.adapter.clone())
    }

    pub fn remove_adapter(&self, host_id: &HostId) -> Option<Rc<OwlenWindowAdapter>> {
        self.adapters
            .borrow_mut()
            .remove(host_id)
            .map(|entry| entry.adapter)
    }

    pub fn has_adapter(&self, host_id: &HostId) -> bool {
        self.adapters.borrow().contains_key(host_id)
    }

    pub fn adapter_count(&self) -> usize {
        self.adapters.borrow().len()
    }

    pub fn pending_creation(&self) -> Option<PendingWindowCreation> {
        self.pending_creation.borrow().clone()
    }
}

/// Custom Slint platform backed by RenderBridge.
pub struct OwlenPlatform {
    bridge: Rc<RenderBridge>,
}

impl OwlenPlatform {
    pub fn new(bridge: Rc<RenderBridge>) -> Self {
        Self { bridge }
    }

    pub fn bridge(&self) -> &Rc<RenderBridge> {
        &self.bridge
    }
}

impl Platform for OwlenPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        let adapter = self.bridge.create_window_adapter()?;
        Ok(adapter as Rc<dyn WindowAdapter>)
    }
}
