# Initial Slint baseline: handoff for a future base-planning phase

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

2026-11-05. [ADR-0006 is Accepted][adr6] by the maintainer. This is a documentation-only handoff, **not approval to begin base-architecture planning, implementation, migration or new execution**. [Authority record][decision], [portability requirements][portability].

## Decisions already made

- Slint is the initial Linux/Wayland renderer, with the existing software/`wl_shm` integration as the concrete baseline. A native CPU client-rendering path is required; acceleration is optional for basic functionality.
- OwlenShell/SCTK retains native connection, dispatch, output, role, surface-generation, placement, input-routing and lifecycle policy. Domain state and persistence remain outside renderer-specific UI state; Core coordinates scheduling through the narrow integration boundary.
- One presenter per surface generation, invalidation before teardown, stale-work rejection and UI-incarnation semantics must survive subsequent design. No general-purpose renderer-plugin runtime is required.
- Multiple Linux distributions, Wayland compositors and GPU/CPU vendors are portability goals, with explicit capability-aware optional integrations. No distribution/hardware certification or non-Linux/X11 commitment follows.
- egui remains a viable preserved alternative; GPUI and incompletely evaluated candidates are not ruled technically unsuitable. The initial selection can be revisited through a documented maintainer decision.

## Evidence available and limits

The [post-S6 assessment][assessment] maps actual Core versus renderer coupling and links the exact S1–S6 revisions. Both renderers pass their tested scopes; overall coverage stays PARTIAL. [S6][s6] includes output-global loss/recreation, controller retention, five natural graceful sessions per renderer and separate fault/abrupt controls. These are private SwayFX results, not universal compositor/physical-hotplug acceptance. Niri multi-output stays BLOCKED.

The ordinary [application entry][prod] still uses Slint composition; the fixture's event-driven scheduling, signal/shutdown and multi-pair orchestration are not a completed production-base migration. Existing domain/output/placement/persistence and custom Slint integration are reusable evidence, not permission for a rewrite. Current exploratory CPU/memory results remain permanently non-decision-grade.

## Open design questions for a separately authorized plan

- Which distribution/compositor/version, hardware/CPU architecture and first usable feature set will have explicit support evidence first? What minimum protocols/dependencies are mandatory, and what optional-feature degradation is acceptable?
- How should the proven narrow boundary and domain/controller ownership appear in the real composition/configuration without copying fixture-only controls or adding a generic renderer framework?
- What asset/font/runtime-path, packaging, dependency/licensing and optional service policies are portable? Which UI/input/accessibility/visual requirements need additional acceptance?
- Is runtime UI interpretation/reload wanted at all? `slint-interpreter`, UI/theme reload, state migration and rollback remain unvalidated; no configuration or reload mechanism is selected.
- Which preserved validation gaps are prerequisites for the declared product scope, and how will production acceptance be evidenced safely? Native fractional buffers, focused removal, controlled wire races and physical/hardware coverage remain open.

## Prerequisites and stop boundary

A maintainer must separately authorize base-architecture planning and its bounded scope. Any later implementation, candidate experiments, benchmarks or production change require their own approval. The accepted initial renderer decision does not pass Gate P or change benchmark guards; [the selection-timing amendment][decision] is limited to this initial architecture choice.

Use the portability/invariant and gap registers as inputs, retain the original checkout and all renderer evidence, and distinguish requirements from tested support in the next proposal. This handoff defines no API, work breakdown, implementation order, estimate or deployment action. Stop here and await the maintainer.

[adr6]: ../../ADR/0006-renderer-selection.md
[decision]: decisions/2026-11-05-initial-slint-selection.md
[portability]: ../../PORTABILITY-REQUIREMENTS.md
[assessment]: POST-S6-DECISION-READINESS.md
[s6]: results/2026-11-05-s6-lifecycle/README.md
[prod]: SOURCE-REFERENCES.md#source-9c0846679bfdd261
