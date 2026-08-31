pub use crate::domain::placement::*;
pub use crate::runtime::placement_engine::*;

/// Backwards compatibility alias for PoC code and tests
pub type ResolveResult = PlacementDecision;

/// Backwards compatibility alias for PoC code and tests
pub type ResolveContext = PlacementContext;

/// Backwards compatibility wrapper for resolve without explicit constraints
pub fn resolve(
    pref: &crate::domain::output::OutputPreference,
    outputs: &[crate::domain::output::OutputSnapshot],
    ctx: &PlacementContext,
) -> PlacementDecision {
    let saved = crate::domain::placement::SavedPlacement {
        output: pref.clone(),
        anchor: Anchor::TopRight,
        offset: (0, 0),
    };
    let constraints = HostPlacementConstraints::default();
    PlacementEngine::resolve(Some(&saved), outputs, ctx, &constraints)
}
