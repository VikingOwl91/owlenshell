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
