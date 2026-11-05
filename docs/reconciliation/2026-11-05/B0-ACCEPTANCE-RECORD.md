# B0 — formal maintainer acceptance record

**B0: FORMALLY ACCEPTED by maintainer on 2026-11-05.**

Acceptance scope: **Base Architecture v1 specifications and contracts only**. This is not implementation completion, runtime validation, production acceptance or B1 authorization. The maintainer explicitly authorized finalizing, signing and publishing the documentation branch. This additive record was prepared for publication on 2026-11-06; the commit/publication date must not be backdated.

## Authority and accepted specifications

Authority is the maintainer's explicit “Finalize B0, Commit & Push” decision. It formally accepts the B0 architecture specification after the documented reviews, rather than inferring acceptance from a reviewer PASS.

- **D01–D11: approved in principle.**
- **P01–P11: approved policy.**
- **K01–K11: accepted B0 contract specifications**, with the refinements and milestone prerequisites recorded below.
- **Generation/token/admission contract: accepted.**
- **Retirement/recreation/shutdown ordering contract: accepted.**
- **Nested Niri: selected as the primary B1 validation strategy.** Exact current private-session/configuration/fixture/binary bindings and execution bounds remain B1 execution prerequisites.

Accepted scope is defined by [MDR-0001](MAINTAINER-DECISIONS.md), [K01–K11](B0-ARCHITECTURE-CONTRACTS.md), the [generation/token specification](B0-GENERATION-TOKEN-CONTRACT.md), the [retirement specification](B0-RETIREMENT-CONTRACT.md) and the [formal acceptance checklist](B0-FORMAL-ACCEPTANCE.md). The [gap disposition](B0-GAP-DISPOSITION.md) and [B1 authorization proposal](B1-AUTHORIZATION-PLAN.md) retain the distinction between an accepted invariant, an unselected engineering mechanism and an unperformed implementation/test.

The accepted architecture retains one native/UI owner thread, SCTK-owned native lifecycle, the concrete Slint software/SHM boundary, one presenter per native generation, separate logical/UI/native/attachment lifetimes, validated queued effects and terminal-close handling, logical state preservation, User-Open Focus, separate protocol/visual observations, configuration/persistence authority, bounded service contracts and ordered local shutdown with explicit stop/restart on complete Wayland loss. Acceptance does not choose unspecified queue capacities, storage mechanisms, public schema details, phase timeouts, provider defaults or an executor.

## Preserved history and unresolved work

The preceding twenty-five reconciliation/consolidation/closure/acceptance-preparation files remain byte-identical. Their draft/proposed/pending-acceptance statements record the state when written. **This record supersedes those acceptance-status statements prospectively for B0 specifications only**; it does not rewrite their findings, review verdicts, evidence or implementation status. No unchecked box in the historical checklist is silently converted into a runtime PASS.

- **All nineteen conflict findings C01–C19 remain OPEN** until their specific implementation, documentation and verification obligations are met. Accepting policy/contracts is not conflict closure.
- **Gate P remains unpassed.**
- **Accepted ADR-0006 remains unchanged.** No accepted ADR is amended and no new ADR is adopted here.
- Historical **S1–S6 evidence, scoped results, limitations, failures and NO-GOs remain unchanged**. Those experiments are not canonical B1 completion or unrestricted production qualification.
- Niri multi-output remains historically **BLOCKED** by the tested private environment. SwayFX results remain compositor-specific.
- Existing performance observations remain **EXPLORATORY — NON-DECISION-GRADE**; no performance ranking or new measurement is introduced.
- Remaining specification gaps retain their [named milestone prerequisites](B0-GAP-DISPOSITION.md#all-fifteen-gaps--specification-disposition). Deferral is not waiver, especially before shared-state access/writes or introduction of a new queue.

## B1 boundary

**B1 implementation and runtime qualification under this accepted base specification have not occurred and are not authorized by this execution.** The maintainer postponed B1 work. Calendar-time passage does not authorize it automatically; a later explicit implementation/runtime decision is required.

The [B1 brief](B1-IMPLEMENTATION-BRIEF.md) and [authorization plan](B1-AUTHORIZATION-PLAN.md) are planning artifacts, not permission to implement their slices, create fixtures, build/run candidates or start Nested-Niri sessions. Installed version/hash discovery in those records does not prove a current operational private validation environment.

No Rust/dependency/accepted-ADR changes, fixtures, compositor experiments, candidate executions, benchmarks, production migration, service integration, PR or merge accompany this acceptance. The publication is limited to the reviewed documentation branch, leaving `main` unchanged.

## Publication evidence

See [publication review](B0-PUBLICATION-REVIEW.md) and [snapshot provenance](B0-PUBLICATION-SOURCES.json). They bind the original inputs and complete reviewed file set, distinguish targeted privacy checks from comprehensive security guarantees, and preserve scanner false-positive triage without changing source evidence.
