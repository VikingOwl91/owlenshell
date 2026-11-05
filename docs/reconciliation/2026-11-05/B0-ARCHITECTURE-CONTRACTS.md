# B0 Architecture Contracts

**Status: proposed operational contracts derived from MDR-0001; implementation not authorized.** 2026-11-05. The maintainer approved D01–D11 in principle and P01–P11 as policy; this document does not itself approve B0 completion or B1 execution. [Decision record](MAINTAINER-DECISIONS.md), [specification gaps](SPECIFICATION-GAPS.md), [milestones and conflict status](CONSOLIDATION-AND-MILESTONES.md).

Every acceptance case below is a **planned test**, not newly executed evidence or a closed conflict. Keep the single package, concrete Slint software/SHM baseline and SCTK-owned native lifecycle. No traits/plugin runtime, async dependency, new crate or direct upstream code reuse is introduced.

## K01: Native/UI owner-thread authority

| Field | Contract specification |
|---|---|
| Invariant | Exactly one application owner thread mutates SCTK/native and Slint/UI objects and canonical runtime state. Work outside that thread returns typed messages through an explicit wake boundary. |
| Owner | App composition/event-loop owner; concrete platform and Slint components execute on that owner thread. |
| Allowed dependencies | Existing calloop/SCTK and concrete Slint integration; neutral domain values; bounded worker messages with the relevant identity. One package is sufficient. |
| Forbidden operations/dependencies | Worker possession of Slint/native handles or `Rc<RenderBridge>`; direct cross-thread Core mutation; a second native dispatch owner or implicitly selected global executor. |
| Failure behavior | Reject invalid/stale delivery before mutation. Failed initialization unwinds acquired resources. A blocking optional provider cannot freeze required native/UI progress or be called successfully stopped without completion. |
| Testable acceptance | Planned tests: owner-thread delivery assertions; fake worker wake→validated application; rejected foreign/stale delivery; no state effect before owner-thread drain; no blocking subprocess in refresh/required startup. Runtime validation requires separate private-environment approval. |
| Related conflicts | C03, C08, C16; all remain OPEN implementation/evidence findings |
| Related milestone | B1; extend wake/completion in B2/B5 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D02, D05; P10; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G03, G12, G15 |

## K02: Core versus Slint responsibilities

| Field | Contract specification |
|---|---|
| Invariant | Core owns the Wayland connection, outputs, layer-shell policy/roles, native tokens/lifecycle and concrete SHM submission. Slint owns renderer UI state, projection, translation, sizing and CPU painting. Exactly one presenter serves a native generation. |
| Owner | WaylandPlatform/SurfaceManager/OutputManager for native authority; concrete Slint bridge for toolkit concerns; App coordinates both. |
| Allowed dependencies | Neutral host state, logical/configured geometry, scale description, current token and presentation admission. Existing RenderBridge/SoftwareRenderer and domain/controller components are reused. |
| Forbidden operations/dependencies | Slint creating/retiring native surfaces or deciding output/placement; Core depending on widget markup; competing SHM/GPU submitters; a speculative multi-renderer/plugin framework or new crate. |
| Failure behavior | Reject duplicate/foreign registration or presentation; surface initialization/render failure is observable and triggers the declared bounded rollback/retirement path, not silent fallback or infinite retry. |
| Testable acceptance | Planned tests: one presenter registration per token; no double attachment; configured size/order before paint; foreign-token rejection; partial-constructor rollback; bar/calendar binding parity; neutral data does not expose toolkit/native objects across the domain seam. |
| Related conflicts | C03, C04, C12; all remain OPEN implementation/evidence findings |
| Related milestone | B1; scheduling integration B2 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D02; established ADR-0003/ADR-0006; P01/P02; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G03, G04, G14 |

## K03: Host/widget identity and canonical state

| Field | Contract specification |
|---|---|
| Invariant | One host registry, one widget/controller registry and canonical validated attachments own logical truth. Logical widget identity and controller state survive native recreation and rehosting. UI incarnations/native tokens may change; reset needs explicit semantics. |
| Owner | Core runtime/Coordinator plus canonical controllers. HostSpec owns desired shell policy, placement resolves it, and SurfaceManager realizes the native interaction boundary. |
| Allowed dependencies | Neutral IDs, validated specs/commands, per-host output context and presentation projections. A Slint UI component may cache a projection but is not a second canonical controller. |
| Forbidden operations/dependencies | Widget ownership of native placement/roles; a duplicate global StateStore or UI truth; incidental month/visibility reset; pretending attachment metadata already materializes arbitrary dynamic Slint children. |
| Failure behavior | Reject missing/duplicate/conflicting attachments before committing composition. Failed native/UI recreation preserves valid logical state and reports materialization failure; do not replace controller state merely to recover presentation. |
| Testable acceptance | Planned tests: one controller/attachment authority; hide/reopen, native-only replacement and explicit rehost preserve identity/data; new UI/native incarnations reject old callbacks; explicit logical reset is distinguishable; two later output compositions remain independent. |
| Related conflicts | C04, C05, C14, C15; all remain OPEN implementation/evidence findings |
| Related milestone | B1; rehost/output B3; desired composition B4 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D03; P03; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G03, G14 |

## K04: Native generations and stale-event rejection

| Field | Contract specification |
|---|---|
| Invariant | Capture relevant native/output/UI lifetime identity at ingress and validate it again when applying an effect. Old native frame/Closed/input effects cannot mutate a replacement. Logical state is not invalidated just because a native surface changes. |
| Owner | Platform captures native identity; App/Coordinator validates effects; UI registration owner validates UI intents. OutputManager and SurfaceManager own their live identities. |
| Allowed dependencies | Existing SurfaceToken/output identity and neutral UI-registration identity where required. Logical controller commands carry only the identity needed for their declared admission semantics. |
| Forbidden operations/dependencies | HostId-only native queues as proof of freshness; enqueue-time-only validation; token reuse based on a matching human-readable output name; blanket fencing that accidentally drops valid already-admitted logical work. |
| Failure behavior | Stale work has no replacement-state/readiness/presentation/domain-action effect. Rejection is observable in tests. The distinction between native-bound events and admitted logical intents must be specified in G03 rather than guessed. |
| Testable acceptance | Planned tests: enqueue old frame/Closed/input, retire/recreate, then drain and assert zero replacement effect; current-token positives still act once; unregister/re-register invalidates old UI intent; validate declared logical-intent behavior across native-only replacement without resetting controllers. |
| Related conflicts | C01, C03, C15, C16; all remain OPEN implementation/evidence findings |
| Related milestone | B1; timer/input/output extensions B2/B3 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D02, D03; P03; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G03, G04 |

## K05: Surface retirement and recreation ordering

| Field | Contract specification |
|---|---|
| Invariant | Fence retiring native admission and invalidate routing/current-token authority before presentation/native release. Cancel native-bound work, release presentation resources according to their lifetime contract, then destroy layer role/surface. Recreate with fresh identities while retaining logical state. |
| Owner | SurfaceManager/native platform, coordinated by App; Slint bridge owns its native-bound presentation/UI-registration resources, not controller truth. |
| Allowed dependencies | Existing route/token registries, explicit native-work cancellation and concrete software resource lifetime. Retain still-valid UI/controller state when replacement is native-only. |
| Forbidden operations/dependencies | Native drop preceding admission invalidation; generation-dependent callbacks accepted after retirement; treating process exit as destruction completion; replacing logical widget state as a cleanup shortcut. |
| Failure behavior | Report incomplete release/destruction separately. Never expose a new active generation with a competing presenter. If construction fails, unwind only acquired resources and keep logical intent available for controlled recovery. |
| Testable acceptance | Planned tests: assert fence→route/token invalidation→work cancellation→presentation release→role/surface destruction ordering; queued callback during transition is rejected; fresh generation reuses logical state; partial-init and repeated-retirement controls. Later native traces corroborate observable ordering without inventing server confirmation. |
| Related conflicts | C01, C02, C10; all remain OPEN implementation/evidence findings |
| Related milestone | B1; shutdown B2; output replacement B3 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D02, D07; P03/P09; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G03, G04, G13 |

## K06: Input dispatch and focus policy

| Field | Contract specification |
|---|---|
| Invariant | Current surface/seat input routes through the native owner and Slint translation to validated Core actions. Explicit clock activation may map an OnDemand calendar under User-Open Focus; the compositor may grant focus immediately, but a grant is not universally guaranteed. |
| Owner | Platform owns native routing/seat/focus facts; Slint bridge translates UI input; Core controllers own semantic actions and input cancellation effects. |
| Allowed dependencies | Current token/seat/registration identity and neutral coordinates/actions. Per-environment contract includes applicable regions, non-text keys, repeat, capture, scroll and seat-loss behavior. |
| Forbidden operations/dependencies | Ignoring stale generations or losing click-without-motion coordinates; duplicate actions; forcing compositor focus; treating missing keyboard capability or focus grant as proof of renderer failure. |
| Failure behavior | Cancel declared pressed/capture/drag/focus state on relevant lifetime loss without duplicate domain effects. Record focus-not-granted or unsupported capabilities distinctly; dependent tests remain UNVERIFIED/BLOCKED as appropriate. |
| Testable acceptance | Planned tests: enter→press→release without motion yields one action; motion/leave and duplicate-native-leave controls; native keyboard focus and supported non-text dismissal; seat/capture loss during interaction; stale-generation input; logical/physical mapping at each later declared scale. Background/unsolicited mapping is not silently covered. |
| Related conflicts | C01, C11, C15; all remain OPEN implementation/evidence findings |
| Related milestone | B1 bounded input; full declared input/output/scale B3 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D04; User-Open Focus; P01/P11; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G01, G03, G05, G14 |

## K07: Frame scheduling and readiness observations

| Field | Contract specification |
|---|---|
| Invariant | App chooses effective wakes/deadlines; Slint reports dirty/toolkit work, and platform owns native configure/frame/buffer admission. Configured, buffer-submitted and callback-observed milestones are independently correlated; visual acceptance is separate. |
| Owner | App scheduler/event loop; native platform for protocol facts; Slint bridge for toolkit deadlines/painting; validation observer for visual findings. |
| Allowed dependencies | Monotonic deadline handling, explicit domain-clock updates, current lifetime identity and existing scoped protocol/visual capture methods. No generic scheduler, compositor diagnostic or proxy is required by this contract. |
| Forbidden operations/dependencies | Unconditional repaint/polling loops, stale deadline effects, no-reason presentation storms, pixel/presentation/input-to-photon claims derived from protocol milestones, or an invented readiness timeout/trigger. |
| Failure behavior | Timeout or lost admission yields an explicit pending/failure observation, never visual PASS. Retiring work invalidates relevant deadlines; required clock/UI changes must not be lost or duplicate domain actions. |
| Testable acceptance | Planned tests: fake-clock deadline expiration/cancellation/replacement; idle→input/state change→required paint→idle; expected clock work distinguished from unintended activity; configure/submission/callback correlated to one current generation; visual verification recorded independently in the selected environment. |
| Related conflicts | C09, C16; all remain OPEN implementation/evidence findings |
| Related milestone | B2; environment/scale extension B3/B6 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D04, D05; P02/P11; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G01, G02, G03 |

## K08: Configuration validation and precedence

| Field | Contract specification |
|---|---|
| Invariant | Initial human-readable configuration is TOML with an explicit schema version and shared validation/application path. Explicit configuration > permitted saved runtime intent > defaults. Runtime topology resolution may use fallback without changing the winning desired intent. |
| Owner | Core configuration authority validates desired host/widget/attachment/service/theme state; Coordinator applies valid desired revisions; persistence supplies only permitted intent. |
| Allowed dependencies | Neutral typed schema/specs and existing pure placement rules. Future GUI/IPC may call the same path, but neither transport is required in the initial base. |
| Forbidden operations/dependencies | Implicit widget-native policy, unknown field/presence semantics treated as settled, saved intent overriding explicit values, topology fallback becoming preferred home, automatic schema migration or an assumed schema-version number. |
| Failure behavior | Invalid parse/validation or failed application preserves the prior valid active revision or reports explicit compensated/degraded outcome. If a valid change was applied but saving fails, P07 permits it to stay active with unsaved/error status; do not mislabel persistence failure as failed application. |
| Testable acceptance | Planned tests: precedence across explicit/permitted-saved/default values, including validated false/zero/empty cases once field semantics are specified; invalid attachment/schema rejected; no partial desired revision; explicit preferred output missing→runtime fallback→return home does not rewrite intent; successful apply plus save failure leaves truthful separate outcomes. |
| Related conflicts | C06, C12, C13, C14; all remain OPEN implementation/evidence findings |
| Related milestone | B4; shared validation constrains later B5 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D06, D08; P04/P05/P07/P08; ADR-0004; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G06, G07, G08, G10, G11 |

## K09: Single-writer persistence and error reporting

| Field | Contract specification |
|---|---|
| Invariant | Only an instance holding verified writer authority may mutate the protected runtime state files. Preserve recoverable last-valid persisted state; no automatic migration or future-version overwrite. Application success, publication to storage and durability acknowledgement are separate outcomes. |
| Owner | Persistence repository owns file operations/results; App owns writer-authority lifetime and dirty/unsaved reporting. Configuration chooses permitted intent, not a second competing writer. |
| Allowed dependencies | Existing versioned DTO/repository logic and an approved exclusivity mechanism for a defined file set. A private single-run test store is not proof of a general user-state writer guard. |
| Forbidden operations/dependencies | Writes/quarantine/repair after denied or unknown authority; general IPC inferred from writer exclusion; default-save/shutdown clobber after future-version fallback; silent overwrite or unreported durability uncertainty. |
| Failure behavior | Pre-publication failure preserves last-valid bytes. A validated applied change may remain active and unsaved. Post-publication durability error must be reported as its actual stage, not falsely assert the old file still occupies the path; G10 must define recoverable preservation/commit guarantees before B4. Unknown/corrupt-state fallback cannot silently enable ordinary writes to the protected source. |
| Testable acceptance | Planned tests with private files/processes: competing writers cannot both mutate; rejected authority causes no state operation; fault injection before write/rename and at selected durability stages; byte-preserved unsupported future file after changes and shutdown; explicit unsaved→retry/success states; corrupt fallback/repair protection; no active user-state paths. |
| Related conflicts | C13, C19; all remain OPEN implementation/evidence findings |
| Related milestone | B4; applies before any earlier shared-state write; B2 shutdown respects protected repository outcomes |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D08, D10; P05–P08; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G06, G08, G09, G10, G11, G13 |

## K10: Service messaging and cancellation

| Field | Contract specification |
|---|---|
| Invariant | Optional shared providers have stable typed state and distinct run identities. Messaging is bounded; cancellation/completion and typed command outcomes are explicit. Replaceable snapshots may coalesce; significant commands/errors are never silently discarded. |
| Owner | Service owner controls producer/run/connection lifetime; App admits messages and projects state; worker never mutates native/UI/Core state directly. |
| Allowed dependencies | Concrete typed requests/results, bounded wake-capable messages, declared coalescing and retry rules. Independently implemented upstream concepts only; executor/provider defaults remain unselected. |
| Forbidden operations/dependencies | Unbounded queues, coalescing toggles/errors as if they were snapshots, false completion on future drop, hidden name takeover, UI-thread waits on unbounded external I/O, or copying unlicensed upstream code. |
| Failure behavior | Overload gives an explicit admission/result/status or visible provider failure, not silent loss. Completion records actual termination/registration release; cancellation does not undo an already-sent external side effect. Stale results are fenced. Missing provider/permission/name conflict is not an automatic transient retry. |
| Testable acceptance | Planned tests: multiple consumers share one provider; defined overload/admission outcomes; latest-value coalescing does not lose commands/completions/errors; cancellation during quiet/blocking I/O wakes and completes; old run cannot write or release a new run’s resources; connection recovery replaces stale connection. Use fakes/private bus only under separate authorization. |
| Related conflicts | C08, C10, C16; all remain OPEN implementation/evidence findings |
| Related milestone | B5; owner-thread/shutdown seams B1/B2 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D05, D11; P10; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G12, G13, G15 |

## K11: Ordered shutdown and complete Wayland loss

| Field | Contract specification |
|---|---|
| Invariant | Fence new work, cancel/complete owned work, retire native-bound UI/presentation before native objects, finish allowed persistence/transport operations and release remaining owners. Complete Wayland connection loss stops the application through ordered local shutdown; no in-process reconnect or automatic restart supervision. |
| Owner | App shutdown state machine coordinates Core, platform, bridge, repository and future workers; each reports its own acquired-resource completion/failure. |
| Allowed dependencies | Existing ownership graph, idempotent phase transitions, bounded completion design and explicit repository outcomes. Attempt protocol flush only while the connection is usable. |
| Forbidden operations/dependencies | Waiting for impossible server acknowledgement on a lost connection; reconnecting behind the policy; treating forced harness cleanup as graceful shutdown; reporting success after silent worker abandonment or unknown persistence outcome. |
| Failure behavior | Report local release, unavailable transport/server confirmation, worker completion and unsaved/error state separately. Apply failure and persistence failure remain distinct. Repeated requests do not duplicate writes/actions. Partial startup unwinds only acquired resources; deadline failure stays incomplete/failed under G13, not fabricated cleanup success. |
| Testable acceptance | Planned tests: normal and partial-init shutdown, repeated requests, calendar open/pending work, output replacement, admitted/stale late events, save failure, worker timeout and actual complete connection loss. No new connection is attempted; no automatic relaunch occurs. Private runtime evidence distinguishes local cleanup from wire destruction confirmation and harness termination. |
| Related conflicts | C01, C02, C07, C10, C16; all remain OPEN implementation/evidence findings |
| Related milestone | B2; extend through B3/B4/B5 |
| Decision/evidence basis | [MDR-0001](MAINTAINER-DECISIONS.md): D07, D09; P07/P09/P10; [original conflict/source evidence](CONFLICT-REGISTER.md) and [original proposal](BASE-ARCHITECTURE-V1-PROPOSAL.md) |
| Remaining specification | [Gap register](SPECIFICATION-GAPS.md): G02, G04, G09, G10, G11, G13 |

## Cross-contract checks

- K08 application transaction success does not imply K09 persistence success. Invalid configuration/application failure differs from P07's valid active-but-unsaved state.
- K09 future-schema write inhibition must survive K11 normal/error shutdown. “Fallback to defaults” is not permission to overwrite the original file.
- K03 logical state survives K05 native/UI replacement. K04 rejects effects using the relevant lifetime, not every logical intent indiscriminately; G03 defines admission precisely.
- K07 protocol observations never become K06/visual acceptance by inference. P01 selects scope, not a compositor, timeout or universal compatibility.
- K10 cancellation must complete under K11 or report failure. No executor/provider defaults, automatic restart or general IPC are approved by these contracts.

The preserved findings identify current gaps; K specifications define the planned target. Accepted ADR-0006, Gate P, original S1–S6 limitations and historical NO-GOs are unchanged.
