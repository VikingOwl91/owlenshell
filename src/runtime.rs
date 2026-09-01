pub mod coordinator;
pub mod placement_engine;
pub mod poc_compat;

pub use crate::model::SurfaceKind;
pub use coordinator::{Coordinator, DragState, LogicalHostState};
pub use placement_engine::PlacementEngine;
pub use poc_compat::{
    bar_output_id, compute_attach_geometry, is_bar_host, is_calendar_host, surface_spec_for,
    AttachGeometry, HANDLE_HEIGHT,
};
