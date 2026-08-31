# ADR-0004: Separate Saved and Runtime Placement

- Status: Accepted

## Context

A preferred monitor may disappear temporarily.

If fallback placement overwrites persistent state, the user's intended location is lost.

## Decision

Represent persisted placement intent and current runtime placement as separate state.

Topology fallback must never automatically persist as user intent.

## Consequences

Positive:

- automatic return-home
- robust docking/undocking
- predictable user intent
- testable pure placement logic

Negative:

- runtime must reconcile two placement states
- diagnostics must expose why current placement differs from saved placement
