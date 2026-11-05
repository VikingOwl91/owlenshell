# ADR 0006: Initial renderer selection

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/ADR/0006-renderer-selection.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../bakeoffs/renderer/SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../bakeoffs/renderer/provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Status: **Accepted — 2026-11-05**, by explicit maintainer decision ([decision and authority record][decision]). Documentation baseline: signed `5951a5f1bae5e1110a1967098d9dc60877e97770`. This accepts an initial architecture choice, not production readiness, implementation or migration.

## Context

OwlenShell is an independent Rust-first Linux/Wayland shell with SCTK-owned native surfaces and an existing Slint software-rendered reference. The bake-off investigated several frontends and subsequently validated Slint and native egui/wgpu against scoped S1–S6 shell behavior. Both demonstrated the native ownership boundary; overall functional coverage remains PARTIAL. There is no decision-grade performance ranking. [Post-S6 assessment][assessment], [S1–S6 matrix and risks][s6], [evidence framework][adr7].

This decision confirms and scopes [ADR-0001's Slint baseline][adr1] after the later investigation. It retains [Core-owned Wayland][adr3], [placement separation][adr4] and [optional compositor adapters][adr5]. Slint is the implementation baseline, not an irrevocable commitment or a claim of universal superiority over egui, GPUI, GTK4, Qt6/QML, Iced or other frameworks.

## Decision and authority

**The maintainer selects Slint as the initial renderer for OwlenShell's first production-oriented base architecture, targeting Linux with Wayland.** Retain the existing software renderer and Core `wl_shm` presentation as the concrete initial baseline. GPU acceleration may be evaluated later but must not become an unconditional requirement for basic shell functionality.

For this initial choice only, the explicit maintainer decision prospectively supersedes this ADR's former deferral pending acceptance/matched benchmarks and the prior Gate P prerequisite before renderer selection. This limited selection-timing amendment is explicit; it is not an assertion that the old prerequisites were met. The earlier Proposed ADR is preserved in Git at the baseline above, and the [pre-decision review][assessment] remains unchanged as a dated record.

**Gate P remains NO-GO/not passed.** Complete production acceptance and separate maintainer approval remain required before production integration or deployment. Gate R eligibility, benchmark guards, separate execution authorization and all historical NO-GOs are unchanged. This selection supplies no benchmark permission and promotes no exploratory result. The general selection prerequisite is not removed for other future choices; any reconsideration must have its own documented maintainer decision. [Current policy notice and original gates][gates], [decision record][decision].

Only documentation is authorized now. Base-architecture planning, implementation, production migration, new renderer experiments and benchmarks await separate approval. [Planning handoff][handoff].

## Decision drivers and rationale

### Existing native ownership compatibility

Slint demonstrated integration behind the narrow OwlenShell-owned SCTK boundary. Core retains the connection/event dispatch, output registry/identities, layer-shell policy, native surfaces/generation tokens, placement/lifecycle, input routing, Coordinator/host policy, persistence and event-loop scheduling authority. Renderer-specific UI state, translation, sizing, painting and toolkit timers remain integration concerns. The fixture uses real domain controller types outside either renderer; its SharedModel/composition is not already a deployed production base. [Ownership/coupling review][assessment], [contracts][ui-contract], [production entry][prod-entry].

### Concrete CPU/software baseline

The tested Slint path paints on the CPU and submits CPU-backed Wayland `wl_shm` buffers through Core. It does not need a Vulkan-capable client GPU to paint that UI. This fits the maintainer's requirement that basic shell rendering have a functional CPU path. It does **not** certify every CPU architecture, distribution, driver, compositor or a wholly GPU-free desktop: the compositor has its own presentation requirements. A software Vulkan implementation such as lavapipe is a GPU-API path and is not equivalent to this native CPU/`wl_shm` baseline. [Software paint][paint], [Core shm path][shm], [portability requirements][portability].

Known visual/feature limits remain explicit: S4A uses integer buffers with compositor fractional downscaling; softness and font/rasterization differences are recorded. Native fractional buffers, authenticated glyph/fallback selection, full alpha-buffer evidence and broad Slint UI-feature coverage are not established. The custom adapter also does not reproduce the prior private developer full-speed rendering-metrics behavior. These are separate from the fact that a software path works in its tested scope. [S4A][s4], [custom-adapter closure][calendar], [validation gaps][portability].

### Scoped functional evidence

Both Slint and native egui passed the declared scopes: real Coordinator/domain bar/clock/calendar composition and actions; pointer, keyboard and scroll controls under the adopted User-Open Focus contract; redraw/deadline/functional idle behavior; five-scale compatibility; independent output pairs in private SwayFX; generation replacement/output-global loss and recovery; application-controlled shutdown and ordered native/resource retirement. S6 records seven scoped scenarios per renderer; fault and SIGKILL controls are not graceful cycles. [S1][s1], [adopted S2][s2], [S3][s3], [S4A][s4], [S5][s5], [S6][s6].

These are successive experiments with distinct source/binary/harness bindings, not unrestricted production acceptance or complete scene/toolkit equivalence. Overall coverage is PARTIAL. Niri multi-output remains BLOCKED by the private test environment; SwayFX evidence cannot close that Niri gap. Historical S2 focus failures, failed captures, initialization failures and oracle corrections retain their recorded outcomes. [Consolidated scope/provenance][assessment].

### Reduced initial migration and integration risk

The maintainer chooses to retain the existing Slint reference, declarative bar/calendar components and working software presentation profile. This avoids an immediate UI-renderer migration and a mandatory GPU-API presentation dependency for the initial base when no demonstrated product-level benefit justifies that migration. It is a maintainer judgment about implementation work, risk and product requirements, **not a measured performance or productivity conclusion**. Custom Slint platform, window, timer, input and buffer integration still require maintenance; production composition is not completed. [Concrete reuse and coupling][assessment], [Slint component reuse][slint-wrapper].

### Declarative UI and possible future runtime configuration

Existing Slint UI is declarative. Runtime interpretation, UI hot-reload, theme reload and state migration may be useful future investigations. OwlenShell integration of `slint-interpreter` and these reload paths has **not been validated**. They are neither implemented product features nor proven deciding capabilities. The accepted choice does not commit a reload mechanism, configuration format or state-migration design. [Current UI reference][slint-wrapper], [open planning questions][handoff].

## Product portability and architectural invariants

The normative [portability requirements][portability] cover multiple Linux distributions, Wayland compositors, GPU vendors and supported CPU architectures. They are architectural goals and compatibility obligations, not a validated-support list. Linux/Wayland is the initial OS scope; Windows, macOS, BSD and X11 are outside this commitment.

The next base must retain Core-owned native lifecycle, one presenter per surface generation, token/registration-safe rejection of stale work, separate domain/UI state, narrow capability-aware compositor integrations, an optional-GPU baseline and explicit predictable optional-capability degradation. Avoid premature renderer-plugin infrastructure. Preserve a practical path to reconsideration without requiring a speculative multi-renderer runtime. The portability document distinguishes already tested properties from future production requirements; it does not declare a fully toolkit-free Core or deployed production seam.

## Alternatives retained

| Alternative | Recorded technical status | Why not the initial baseline / future conditions |
|---|---|---|
| Native egui/wgpu |Technically viable alternative. Same Core/SCTK ownership and scoped S1–S6 passes, including SwayFX multi-output/lifecycle. Surface-format defect diagnosed and corrected in the experiment. |The current presenter uses a Vulkan GPU-API rendering/presentation path; a native CPU-only presentation fallback has not been designed/validated. Adapter/device/surface capability handling adds initial integration requirements. No demonstrated product benefit currently justifies leaving Slint. This is not a rejection for functional failure, a physical-GPU-vendor requirement or proof egui fundamentally cannot support CPU rendering. [Native evidence][assessment], [format correction][format]. |
| Historical wayapp-egui |Archived feasibility/input/performance work with its own native ownership/input integration. |Not the current native Core alternative. Preserve its pointer-enter defect and original evidence; do not transfer that defect to egui core or erase it because the new integration works. [Historical review][old-review]. |
| GPUI / gpui-shell reference |Future research candidate; has not passed OwlenShell's SCTK ownership and S1–S6 gates. |An existing gpui-shell project does not establish OwlenShell compatibility. No claim of definitive unsuitability, new experiment authorization or architectural foundation follows. |
| GTK4, Qt6/QML, Iced |Preserved prototypes/feasibility and partial acceptance records; no S1–S6 investigation for these candidates. |Not selected initially. Their incomplete evidence is not a technical rejection or performance ranking. [Candidate matrix][matrix], [bake-off index][index]. |
| COSMIC/libcosmic |**WITHDRAWN — NOT FULLY EVALUATED** by maintainer decision. |Preserve source/build/feasibility evidence; not technically rejected or failed. [Index][index]. |

All experimental implementations remain in the repository. Existing CPU/memory observations remain **EXPLORATORY — NON-DECISION-GRADE**, from older workloads/integrations; later adapter metadata must not be retroactively attributed to the original idle runs. No decision-grade performance comparison is available. This ADR makes no speed, memory, efficiency or universal visual-quality ranking. [Original observations and limits][exploratory], [post-run analysis][exploratory-analysis], [ADR-0007][adr7].

## Consequences and outstanding acceptance

Positive consequences: a concrete implementation baseline, reuse of known UI/Core integration, a native software client path and a bounded direction for the next planning request. Costs and risks remain: custom Slint integration maintenance, software buffer/conversion work, visual/feature limits, portability/dependency/font/packaging/licensing obligations, and incomplete supported-system acceptance. No estimates of comparative developer effort, build footprint or runtime cost are inferred.

The outstanding [validation register][portability] preserves Niri multi-output BLOCKED; keyboard-focused removal, controlled native-wire races, physical hotplug, native fractional buffers, complete distribution/hardware coverage and runtime reload UNVERIFIED; production integration/acceptance NOT PERFORMED; and decision-grade performance comparison NOT AVAILABLE. Duplicate native pointer-leave observations stay PARTIAL at protocol-event level with scoped idempotent semantic cancellation/action findings; nothing is suppressed to obtain a PASS. [S5][s5], [S6][s6].

## Reconsideration

Slint is the initial baseline, not an irrevocable technology commitment. Revisit through a new documented maintainer architectural decision if requirements/evidence materially change: unsupported required UI/accessibility/visual features; new portability or dependency/licensing constraints; maintenance problems; significant regressions; new native-surface/fallback integration; or reviewed product/performance evidence establishing a reason to change. Preserve prior prototypes, failed results and decision rationale. Do not infer automatic adoption from a new release, GPU acceleration or another project's shell. Future integration and execution permissions remain separate.

[decision]: ../bakeoffs/renderer/decisions/2026-11-05-initial-slint-selection.md
[portability]: ../PORTABILITY-REQUIREMENTS.md
[handoff]: ../bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md
[assessment]: ../bakeoffs/renderer/POST-S6-DECISION-READINESS.md
[s1]: ../bakeoffs/renderer/results/2026-11-04-s1-functional-parity/README.md
[s2]: ../bakeoffs/renderer/results/2026-11-05-s2-user-open-focus/README.md
[s3]: ../bakeoffs/renderer/results/2026-11-05-s3-scheduling/README.md
[s4]: ../bakeoffs/renderer/results/2026-11-05-s4-scaling/README.md
[s5]: ../bakeoffs/renderer/results/2026-11-05-s5-multi-output/README.md
[s6]: ../bakeoffs/renderer/results/2026-11-05-s6-lifecycle/README.md
[gates]: ../bakeoffs/renderer/GATE-PROPOSAL.md
[adr1]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-7bfab1482e925315
[adr3]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-fafd874e426c2eeb
[adr4]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-75fa4debaa46f91a
[adr5]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-eea96193892d525f
[adr7]: 0007-renderer-bakeoff-evidence-scope.md
[paint]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-44baccb73b186f1d
[shm]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-81cd532c3d0525f8
[ui-contract]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-e2a9690cc59100a9
[prod-entry]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-9c0846679bfdd261
[slint-wrapper]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-829c5ac42f4f1d1a
[calendar]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-e7c0480d239b7289
[format]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-590bbbadab5ec52e
[old-review]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-e965c84ec042341d
[matrix]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-4f8ae4cfe1184b81
[index]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-ee9aeb2c5b99b9d5
[exploratory]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-650d9c951033d2ac
[exploratory-analysis]: ../bakeoffs/renderer/SOURCE-REFERENCES.md#source-a41eff4735c296d2
