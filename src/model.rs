use serde::{Deserialize, Serialize};
use smithay_client_toolkit::shell::wlr_layer::Anchor as LayerAnchor;

pub const CALENDAR_KEY: &str = "calendar";
pub const MIN_VISIBLE: i32 = 32;
pub const BAR_HEIGHT: u32 = 32;
pub const HANDLE_HEIGHT: f64 = 28.0;
pub const BTN_LEFT: u32 = 0x110;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Anchor {
    pub fn as_layer_anchor(self) -> LayerAnchor {
        match self {
            Self::TopLeft => LayerAnchor::TOP | LayerAnchor::LEFT,
            Self::TopRight => LayerAnchor::TOP | LayerAnchor::RIGHT,
            Self::BottomLeft => LayerAnchor::BOTTOM | LayerAnchor::LEFT,
            Self::BottomRight => LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPreference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_size_mm: Option<(i32, i32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPlacement {
    pub output: OutputPreference,
    pub anchor: Anchor,
    pub offset: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct OutputSnapshot {
    pub id: u32,
    pub name: Option<String>,
    pub make: String,
    pub model: String,
    pub physical_size_mm: (i32, i32),
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub scale: i32,
    pub serial: Option<String>,
    pub preferred: bool,
}

impl OutputSnapshot {
    pub fn to_preference(&self) -> OutputPreference {
        OutputPreference {
            name: self.name.clone(),
            make: empty_to_none(&self.make),
            model: empty_to_none(&self.model),
            physical_size_mm: Some(self.physical_size_mm),
            serial: self.serial.clone().filter(|s| !s.is_empty()),
        }
    }
}

fn empty_to_none(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Bar,
    Calendar,
}
