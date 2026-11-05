# OwlenShell architecture reconciliation

**2026-11-05 · Draft for maintainer review · Documentation only**

Canonical baseline: `0cb96dcdbe0f94f35db3f8d30ab91765402ba000` on `main` (PR #2 merged). Local branch: `docs/architecture-reconciliation`. Upstream reference: `andre-brandao/gpui-shell` at `06616a02662621af84f0841dba3c48db1560bc82`. GPL-3.0-only and Accepted ADR-0006 are present at this baseline. No application/dependency/ADR/evidence changes, implementation, migration, new runtime experiment, benchmark, merge, push or PR are part of this pass.

## Assessment

**The existing architecture is reusable; production boundary reconciliation is still required.** The canonical application has real SCTK-owned surfaces, Slint software/SHM presentation, typed host/widget policies, Coordinator/controllers, output tracking, pure placement and versioned persistence. It is one Rust package, not a service/plugin framework. No rewrite is justified by this audit.

The crucial distinction is between that application, the separate S1–S6 experiment and future product requirements. Slint and native egui passed their declared experimental scopes, with overall PARTIAL coverage; the extracted UiRuntime and lifecycle fixes are not already canonical code. Slint remains the maintainer-selected initial renderer. Gate P is unpassed, Niri multi-output remains environment-BLOCKED, SwayFX evidence stays compositor-specific, and exploratory observations are not a performance ranking.

## Deliverables

1. [Implementation/evidence inventory](IMPLEMENTATION-EVIDENCE.md): current responsibilities, source/tests, missing functions and evidence boundaries.
2. [Conflict and risk register](CONFLICT-REGISTER.md): C01–C19, concrete sources, severity, resolution and ADR/approval need.
3. [GPUi Shell Linux-integration study](GPUI-LINUX-REFERENCE.md): sixteen source-grounded concept/provider assessments, licensing limits and recommendations.
4. [Proposed Base Architecture v1](BASE-ARCHITECTURE-V1-PROPOSAL.md): small cohesive single-package boundaries, ownership diagram, lifetimes, scheduling/services/config/shutdown contracts.
5. [Milestone and gate proposal](MILESTONES-AND-GATES.md): B0–B6 dependencies, tests, acceptance, environments and recovery.
6. [Open maintainer decisions](OPEN-DECISIONS.md): D01–D11; no new decision is silently accepted.
7. [Independent review](REVIEW.md) and [source/provenance manifest](SOURCES.json).

## Priority reconciliation outcomes

- **C01/C02:** platform enqueue-time validation does not ensure end-to-end stale-event rejection. Coordinator ignores lifecycle tokens, pending input discards generation and retirement code differs from its invalidation-before-release comment. These are static source gaps, not newly reproduced native races. SCTK itself handles configure acknowledgements and layer-role destruction correctly in inspected code.
- **C03/C05/C14:** state and renderer separation is incomplete. Slint types still cross controller/Coordinator/geometry boundaries, HostState references platform topology, attachments are not a materialized widget registry and UI construction lives in main. Reuse the core mechanisms and extract only the concrete seam.
- **C07/C08/C10:** documents overstate session integration; UWSM paths are no-ops/unsupported. Compositor enrichment is synchronous and unbounded, and application-wide shutdown fencing/error cleanup is not implemented. These must not be described as production capabilities.
- **C04/C06/C12/C13:** clarify host policy versus native realization, desired config versus runtime intent, canonical state and one geometry authority. Existing atomic persistence is narrower than durable/concurrent transaction guarantees.
- **C15–C19:** current main has one fixed pair, incomplete input/focus/deadline contracts and non-hermetic default-path testing. Dated stage plans, old PoC claims and experimental statuses cannot substitute for current acceptance. Base Architecture v1 is not a redefinition of full Product v1.

These recommendations do not weaken historical contracts. [ADR-0006](../../ADR/0006-renderer-selection.md), [portability requirements](../../PORTABILITY-REQUIREMENTS.md) and the [Slint handoff](../../bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md) remain unchanged.

## External reference conclusions

Useful concepts: stable shared provider state, typed commands, run identities, explicit eager/lazy/off policy, snapshot-to-stream handoff and explicit exclusive-name ownership. Adapt these with bounded admission, cancellation completion, cache recovery and Core-thread message delivery. Do not adopt GPUI native ownership, mandatory compositor-specific startup, unbounded queues/IPC or detached-worker shutdown assumptions.

Audio control is libpulse, not a native PipeWire mixer; PipeWire is used separately for privacy monitoring. Notification and tray stop/name ownership differ. Upstream source existence is not tested OwlenShell compatibility. No general project license was found at the pinned revision; the icon license is asset-scoped. **Direct source reuse remains constrained pending explicit applicable permission/license.** This study copies no upstream code or assets and changes no dependency.

## Validation and honesty boundaries

Fresh current-code tests: **115 passed, 0 failed, 0 ignored**, with private HOME/XDG/TMPDIR and display/compositor/session-bus environment scrubbed. The earlier default-state-path invocation is retained but not used as isolation evidence; strict rerun created its state only in the private directory and existing user-state file metadata remained unchanged during that rerun. This is recorded as C19, not hidden by the passing count. No compositor/candidate/upstream executable or service was run.

Static validation checks source references/symbol locations, local links/anchors, pinned upstream paths/line ranges, metadata, privacy, unchanged historical evidence and exact documentation-only scope. It does not certify live buffer/pixel/focus/physical-hotplug/production behavior. Source observations, prior experimental observations and proposals use separate labels throughout. Full original experiment/raw artifacts are not republished; their retained source identities are not public reproduction promises.

**Proposed next action:** maintainer review of D01–D04/B0, then separate authorization for the smallest B1 native-generation/concrete Slint boundary slice. No implementation starts automatically after this report. All remaining service/config/IPC/security and production gates retain their own approval requirements. Stop here for maintainer judgment.
