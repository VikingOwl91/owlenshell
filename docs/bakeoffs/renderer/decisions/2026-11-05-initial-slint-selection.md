# Maintainer decision: initial Slint renderer

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/decisions/2026-11-05-initial-slint-selection.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Date:2026-11-05. Starting signed commit `5951a5f1bae5e1110a1967098d9dc60877e97770`, clean branch `experiment/egui-wgpu-bakeoff` in the durable experimental evidence repository.

**Maintainer decision: ACCEPT Slint as the initial renderer for OwlenShell's first production-oriented Linux/Wayland base architecture.** This is explicit maintainer authority, not an assistant inference from tests or benchmark results. [ADR-0006](../../../ADR/0006-renderer-selection.md) records the rationale and consequences; [portability requirements](../../../PORTABILITY-REQUIREMENTS.md) record the distribution/compositor/hardware goals; [handoff](../SLINT-BASE-PLANNING-HANDOFF.md) records the boundary for the next phase.

## Limited selection-timing amendment

For this initial Slint architecture choice only, the maintainer's explicit acceptance prospectively supersedes ADR-0006's former deferral pending acceptance/matched benchmarks and the prior Gate P prerequisite before renderer selection. The initial renderer has been chosen despite the explicitly incomplete evidence. It does not mean those earlier prerequisites were satisfied and does not grant a general exception for future renderer choices.

Gate P remains NO-GO/not passed; full production acceptance and separate approval remain required before production integration or deployment. Gate R eligibility, measurement/benchmark guards and separate execution authorization are unchanged. No historical NO-GO, FAIL, scoped PASS or PARTIAL result is reclassified. All CPU/memory observations stay EXPLORATORY — NON-DECISION-GRADE. The [post-S6 assessment](../POST-S6-DECISION-READINESS.md) and previous reviews remain exact pre-decision snapshots in their original scopes. [Dated current policy notice](../GATE-PROPOSAL.md).

## Authority and scope

Authorized now: documentation of the decision, portability requirements, retained ownership invariants, known gaps, a short handoff, independent review, static documentation checks and a signed documentation-only commit.

Not authorized: beginning the base-architecture plan or implementation; production migration; original-checkout changes; deleting/archiving renderer implementations; new candidate execution, measurements or benchmarks; a generic renderer plugin framework. The choice claims neither unrestricted production acceptance nor superiority. Native egui stays technically viable; GPUI is unverified rather than unsuitable; other candidate/withdrawal statuses stay historical.

Original evidence and failures must be preserved. Niri multi-output remains BLOCKED by the private environment; broader distro/compositor/hardware/CPU-only host support and hot-reload remain unverified. The initial choice can be revisited through a new documented maintainer architectural decision.

[Independent review](../reviews/2026-11-05-initial-slint-selection.md). Stop after the documentation handoff and await separate planning/implementation authorization.
