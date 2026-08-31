pub mod output_manager;
pub mod surface_manager;

pub use output_manager::{
    NewOutput, OutputRegistry, TopologyEvent, TopologyGeneration, TopologySnapshot,
};
pub use surface_manager::{
    SurfaceCommand, SurfaceGeneration, SurfaceGenerationRegistry, SurfaceLifecycleEvent,
    SurfaceSlot, SurfaceSpec, SurfaceToken,
};

use crate::domain::ids::{HostId, OutputId};

/// Top-level typed events emitted by the platform layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent {
    Topology(TopologyEvent),
    Surface(SurfaceLifecycleEvent),
}

/// Errors originating from platform operations or state validations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    OutputNotFound(OutputId),
    HostNotFound(HostId),
    StaleGeneration { token: SurfaceToken },
    InvalidOperation(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutputNotFound(id) => write!(f, "output not found: {id}"),
            Self::HostNotFound(id) => write!(f, "host not found: {id}"),
            Self::StaleGeneration { token } => write!(f, "stale surface generation: {token}"),
            Self::InvalidOperation(msg) => write!(f, "invalid platform operation: {msg}"),
        }
    }
}

impl std::error::Error for PlatformError {}
