pub mod calendar_widget;
pub mod clock_widget;
pub mod coordinator;
pub mod placement_engine;
pub mod poc_compat;

pub use crate::domain::default_shell::{
    ShellSpec, WidgetAttachmentSpec, DEFAULT_BAR_HOST_ID, DEFAULT_CALENDAR_HOST_ID,
    DEFAULT_CALENDAR_WIDGET_ID, DEFAULT_CLOCK_WIDGET_ID,
};
pub use crate::domain::host::{HostSpec, HostState};
pub use crate::domain::widget::{HostCommand, WidgetAction, WidgetKind, WidgetSpec, WidgetState};
pub use crate::model::SurfaceKind;
pub use calendar_widget::{days_in_month, CalendarDayRow, CalendarWidgetController};
pub use clock_widget::{clock_text, ClockWidgetController};
pub use coordinator::{Coordinator, DragState, LogicalHostState};
pub use placement_engine::PlacementEngine;
pub use poc_compat::{
    bar_output_id, compute_attach_geometry, is_bar_host, is_calendar_host, surface_spec_for,
    AttachGeometry, HANDLE_HEIGHT,
};
