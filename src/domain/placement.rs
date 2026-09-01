use crate::domain::ids::OutputId;
use crate::domain::output::OutputPreference;

/// Semantic anchor point on an output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

/// Persisted user placement intent.
///
/// Distinct from runtime placement (`PlacementDecision`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SavedPlacement {
    pub output: OutputPreference,
    pub anchor: Anchor,
    pub offset: (i32, i32),
}

/// Explanation for why an output resolution decision was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolveReason {
    /// Exact connector or monitor name matched.
    ExactName,
    /// Strong hardware serial number matched.
    Serial,
    /// Unique hardware fingerprint (make/model/dimensions) matched.
    Fingerprint,
    /// Configured logical output role matched.
    LogicalRole,
    /// Source or spawn output matched.
    SpawnOutput,
    /// Compositor preferred or focused output matched.
    Preferred,
    /// Deterministic fallback output selected.
    Any,
    /// No usable output was available; placement deferred.
    Deferred,
}

/// Attributes of a successfully placed surface on an active output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedSurface {
    pub target_output_id: OutputId,
    pub anchor: Anchor,
    pub offset: (i32, i32),
    pub margins: (i32, i32, i32, i32),
    pub reason: ResolveReason,
    pub is_fallback: bool,
    pub is_reachable: bool,
}

/// Attributes of a deferred surface when no target output can host it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredSurface {
    pub anchor: Anchor,
    pub offset: (i32, i32),
    pub margins: (i32, i32, i32, i32),
    pub reason: ResolveReason,
}

/// Runtime decision produced by the PlacementEngine.
///
/// Strictly distinct from `SavedPlacement`. Contradictory states are unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementDecision {
    Placed(PlacedSurface),
    Deferred(DeferredSurface),
}

impl PlacementDecision {
    pub fn target_output_id(&self) -> Option<OutputId> {
        match self {
            Self::Placed(p) => Some(p.target_output_id),
            Self::Deferred(_) => None,
        }
    }

    pub fn anchor(&self) -> Anchor {
        match self {
            Self::Placed(p) => p.anchor,
            Self::Deferred(d) => d.anchor,
        }
    }

    pub fn offset(&self) -> (i32, i32) {
        match self {
            Self::Placed(p) => p.offset,
            Self::Deferred(d) => d.offset,
        }
    }

    pub fn margins(&self) -> (i32, i32, i32, i32) {
        match self {
            Self::Placed(p) => p.margins,
            Self::Deferred(d) => d.margins,
        }
    }

    pub fn reason(&self) -> ResolveReason {
        match self {
            Self::Placed(p) => p.reason,
            Self::Deferred(d) => d.reason,
        }
    }

    pub fn is_fallback(&self) -> bool {
        match self {
            Self::Placed(p) => p.is_fallback,
            Self::Deferred(_) => false,
        }
    }

    pub fn is_reachable(&self) -> bool {
        match self {
            Self::Placed(p) => p.is_reachable,
            Self::Deferred(_) => false,
        }
    }

    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred(_))
    }
}

/// Resolved attachment geometry for a native surface on a Wayland output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachGeometry {
    pub anchor: Anchor,
    pub margins: (i32, i32, i32, i32),
    pub requested_size: (u32, u32),
    pub effective_widget_size: (i32, i32),
    pub offset: (i32, i32),
}
