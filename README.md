# OwlenShell

OwlenShell is an experimental modular Rust Linux/Wayland desktop shell. The maintainer selected Slint as its initial renderer, with an SCTK-owned native surface lifecycle and a software-rendering baseline.

> **Status: Early Architecture & Prototyping**
>
> Initial proof-of-concepts and scoped integration tests demonstrate architectural feasibility. OwlenShell is not yet a complete or production-ready shell. Most planned system integrations, configurable components and broader compatibility remain future goals.

This branch adds a [curated renderer-decision record](docs/bakeoffs/renderer/README.md) to the canonical main baseline. It does not import the experimental implementation or complete history. See [Accepted ADR-0006](docs/ADR/0006-renderer-selection.md), [portability goals and gaps](docs/PORTABILITY-REQUIREMENTS.md) and the [existing architecture](docs/ARCHITECTURE.md). Required distributions, compositors and hardware are targets, not a validated support list. No new execution, production migration or benchmark is performed here.
