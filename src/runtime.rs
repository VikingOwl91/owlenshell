pub mod calendar_widget;
pub mod clock_widget;
pub mod coordinator;
pub mod placement_engine;

pub use crate::domain::default_shell::{
    ShellSpec, WidgetAttachmentSpec, DEFAULT_BAR_HOST_ID, DEFAULT_CALENDAR_HOST_ID,
    DEFAULT_CALENDAR_WIDGET_ID, DEFAULT_CLOCK_WIDGET_ID,
};
pub use crate::domain::host::{HostSpec, HostState};
pub use crate::domain::placement::AttachGeometry;
pub use crate::domain::widget::{HostCommand, WidgetAction, WidgetKind, WidgetSpec, WidgetState};
pub use crate::model::HANDLE_HEIGHT;
pub use calendar_widget::{days_in_month, CalendarDayRow, CalendarWidgetController};
pub use clock_widget::{clock_text, ClockWidgetController};
pub use coordinator::{Coordinator, DragState};
pub use placement_engine::{compute_attach_geometry, PlacementEngine};
