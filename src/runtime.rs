pub mod calendar_widget;
pub mod clock_widget;
pub mod coordinator;
pub mod placement_engine;
pub mod poc_compat;

pub use crate::domain::host::{HostSpec, HostState};
pub use crate::domain::widget::{WidgetAction, WidgetKind, WidgetSpec, WidgetState};
pub use crate::model::SurfaceKind;
pub use calendar_widget::{days_in_month, CalendarDayRow, CalendarWidgetController};
pub use clock_widget::{clock_text, ClockWidgetController};
pub use coordinator::{Coordinator, DragState, LogicalHostState};
pub use placement_engine::PlacementEngine;
pub use poc_compat::{
    bar_output_id, compute_attach_geometry, is_bar_host, is_calendar_host, surface_spec_for,
    AttachGeometry, HANDLE_HEIGHT,
};
