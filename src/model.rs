pub use crate::domain::*;

pub const CALENDAR_KEY: &str = "calendar";
pub const MIN_VISIBLE: i32 = 32;
pub const BAR_HEIGHT: u32 = 32;
pub const HANDLE_HEIGHT: f64 = 28.0;
pub const BTN_LEFT: u32 = 0x110;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Bar,
    Calendar,
}

impl Anchor {
    pub fn as_layer_anchor(self) -> smithay_client_toolkit::shell::wlr_layer::Anchor {
        use smithay_client_toolkit::shell::wlr_layer::Anchor as LayerAnchor;
        match self {
            Self::TopLeft => LayerAnchor::TOP | LayerAnchor::LEFT,
            Self::TopRight => LayerAnchor::TOP | LayerAnchor::RIGHT,
            Self::BottomLeft => LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            Self::BottomRight => LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
            Self::Top => LayerAnchor::TOP | LayerAnchor::LEFT | LayerAnchor::RIGHT,
            Self::Bottom => LayerAnchor::BOTTOM | LayerAnchor::LEFT | LayerAnchor::RIGHT,
            Self::Left => LayerAnchor::LEFT | LayerAnchor::TOP | LayerAnchor::BOTTOM,
            Self::Right => LayerAnchor::RIGHT | LayerAnchor::TOP | LayerAnchor::BOTTOM,
            Self::Center => LayerAnchor::empty(),
        }
    }
}
