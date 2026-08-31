# ADR-0005: Generic Wayland with Compositor Adapters

- Status: Accepted

## Context

Generic Wayland does not expose every useful shell concept, including universal primary-output semantics, strong physical identity, global geometry or binding introspection.

Hard-depending on one compositor would conflict with Owlenshell's identity.

## Decision

Provide a generic Wayland baseline and optional compositor adapters.

Adapters enrich generic state but do not replace the core runtime.

## Consequences

Positive:

- compositor-agnostic architecture
- richer integration where APIs exist
- graceful degradation elsewhere

Negative:

- capability differences between compositors
- adapter maintenance burden
- some advanced features may not be universally available
