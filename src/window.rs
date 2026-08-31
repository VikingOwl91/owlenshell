use slint::platform::{
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    WindowAdapter, WindowProperties,
};
use slint::{LogicalSize, PhysicalSize, PlatformError, Window, WindowSize};
use std::cell::Cell;
use std::rc::Rc;

pub struct OwlenWindowAdapter {
    pub window: Rc<MinimalSoftwareWindow>,
    pub surface: Cell<Option<usize>>,
    pub preferred: Cell<LogicalSize>,
    pub visible: Cell<bool>,
}

impl OwlenWindowAdapter {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            window: MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer),
            surface: Cell::new(None),
            preferred: Cell::new(LogicalSize::new(32.0, 32.0)),
            visible: Cell::new(true),
        })
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
