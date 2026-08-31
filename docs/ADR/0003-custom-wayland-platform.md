# ADR-0003: Own the Wayland Platform Layer

- Status: Accepted

## Context

The atomic-surface PoC requires shell-specific Wayland semantics, including layer-shell, output assignment, custom placement and controlled surface geometry.

Slint's normal window backend does not provide the required shell behavior directly.

## Decision

Owlenshell owns the Wayland platform using Rust Wayland tooling such as Smithay Client Toolkit and protocol bindings.

Slint is hosted through a custom platform/window adapter.

## Consequences

Positive:

- direct control over shell surfaces
- compositor-independent generic Wayland baseline
- correct geometry/input ownership
- no need to force shell semantics through a desktop-window abstraction

Negative:

- Owlenshell owns event-loop and lifecycle complexity
- scaling/input/render integration requires careful testing
