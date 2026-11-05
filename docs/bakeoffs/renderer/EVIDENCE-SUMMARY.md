# Curated renderer evidence and decision summary

This is a derived reading guide, not an execution receipt or complete reproduction package. The maintainer accepted Slint as the initial Linux/Wayland renderer in [ADR-0006](../../ADR/0006-renderer-selection.md). This is a product/risk judgment, not a performance ranking. GPU acceleration is optional for the initial software-rendered baseline. Production acceptance remains incomplete.

## Unranked candidate status

| Candidate | Established scope and decision treatment | Remaining boundary |
|---|---|---|
| Slint | Selected initial renderer; concrete software rendering/Core wl_shm path; S1–S6 scoped functional passes | Overall PARTIAL, not production acceptance or whole-host GPU-free certification |
| Native egui/wgpu | Technically viable Core/SCTK-owned alternative; S1–S6 scoped passes; advertised UNORM8 format correction validated | Current presenter uses a GPU-API path; native CPU-only fallback not designed/validated; no functional-failure rejection |
| GTK4 | Historical prototype and partial feasibility/acceptance evidence retained locally | Not evaluated through S1–S6; incomplete evidence is not a technical rejection |
| Qt6/QML | Historical prototype and partial feasibility/acceptance evidence retained locally | Not evaluated through S1–S6; incomplete evidence is not a technical rejection |
| Iced | Historical prototype and partial feasibility/acceptance evidence retained locally | Not evaluated through S1–S6; incomplete evidence is not a technical rejection |
| COSMIC/libcosmic | WITHDRAWN — NOT FULLY EVALUATED, by maintainer decision | Not technically rejected or failed |
| GPUI | Future research candidate/reference | OwlenShell native-ownership and parity gates not established; gpui-shell does not establish compatibility |

## Functional scope shared by Slint and native egui

| Slice | Recorded outcome for each renderer | Scope / limits |
|---|---|---|
| [S1](results/2026-11-04-s1-functional-parity/README.md) | PASS tested action/state/lifecycle contract | One output, scale 1; real Coordinator bar/clock/calendar state; 30 normalized snapshots match; not pixel equality |
| [S2](results/2026-11-05-s2-user-open-focus/README.md) | Adopted User-Open Focus scoped validation | Original no-focus-on-map failure/NO-GO preserved; four UNVERIFIED items per renderer remain in report |
| [S3](results/2026-11-05-s3-scheduling/README.md) | Scoped functional scheduling/deadline/idle validation | Observable functional activity, no CPU/FPS/energy claim |
| [S4A](results/2026-11-05-s4-scaling/README.md) | Scoped compatibility at 1, 1.25, 1.5, 1.75, 2 | Integer-buffer/compositor scaling; not native fractional-buffer implementation |
| [S5](results/2026-11-05-s5-multi-output/README.md) | Scoped two-output functional parity | Private headless SwayFX only; duplicate native pointer-leave remains PARTIAL; keyboard/focus limits retained |
| [S6](results/2026-11-05-s6-lifecycle/README.md) | 7/7 scoped scenarios PASS per renderer; overall PARTIAL | Synthetic output loss/recreation and application shutdown, not physical hotplug or complete wire-race coverage |

These are successive experiments with distinct source/binary/harness bindings. Existing performance observations are permanently **EXPLORATORY — NON-DECISION-GRADE**, concern older workloads/integrations and are not a comparison of current native adapters. No numerical performance table is included. No decision-grade performance comparison is available; no scene-equivalence, renderer ranking or Gate R promotion follows.

## Retained architecture

Core owns SCTK connection/dispatch, output identities, layer-shell policy, native surfaces/generations, placement, input routing, Coordinator/domain state and persistence. A narrow UI boundary contains toolkit UI state, input translation, sizing, painting and toolkit deadlines. Exactly one presenter serves a surface generation; stale work is rejected, and GPU resources release before native destruction in the tested native egui path. Fixture orchestration is not already production composition. See [coupling/risks](POST-S6-DECISION-READINESS.md), [portability](../../PORTABILITY-REQUIREMENTS.md) and [planning handoff](SLINT-BASE-PLANNING-HANDOFF.md).

Niri multi-output remains BLOCKED by the tested environment. Keyboard-focused output removal, controlled native-wire races, physical hotplug, native fractional buffers, full distribution/hardware/CPU-only-host coverage and runtime Slint UI reload remain UNVERIFIED. Production integration/acceptance is NOT PERFORMED. Required portability is a goal, not a certified support matrix. Historical failures, corrections, independent disagreements and NO-GOs remain unchanged in the retained original evidence.
