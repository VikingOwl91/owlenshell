# ADR-0001: Use Slint for Presentation

- Status: Accepted

## Context

Owlenshell requires a native UI layer without GTK, Qt, GJS, QML or a browser runtime as the shell foundation.

The UI layer must integrate with a runtime that directly owns Wayland shell surfaces.

## Decision

Use Slint for component layout, presentation, animation and rendering.

Use a custom Slint platform/window adapter where required so Owlenshell retains ownership of Wayland surfaces.

## Consequences

Positive:

- Rust-friendly UI layer
- no GTK/Qt shell dependency
- declarative UI
- suitable for reusable components
- renderer can remain separate from shell semantics

Negative:

- Owlenshell must implement more Wayland/window integration itself
- default winit behavior is not sufficient for all shell surface requirements
