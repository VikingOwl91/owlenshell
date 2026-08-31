use crate::domain::host::HostMovementPolicy;

/// Explicit runtime movement capability provided by the active input/compositor environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovementCapability {
    /// Full global pointer stream allowing continuous cross-surface drag.
    GlobalPointerStream,
    /// Stable relative motion events from input device.
    StableRelativeMotion,
    /// Dedicated compositor move session protocol.
    CompositorMoveSession,
    /// Drag movement is not supported in the current environment.
    Unavailable,
}

impl MovementCapability {
    pub fn is_draggable(&self) -> bool {
        matches!(
            self,
            Self::GlobalPointerStream | Self::StableRelativeMotion | Self::CompositorMoveSession
        )
    }
}

/// Evaluates whether a host can be moved given its declared policy and observed runtime capability.
pub fn can_move(policy: HostMovementPolicy, capability: MovementCapability) -> bool {
    policy == HostMovementPolicy::Movable && capability.is_draggable()
}
