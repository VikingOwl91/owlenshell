# MDR-0001 — Base Architecture v1 Maintainer Decisions

Decision date: **2026-11-05**.
Status: **Maintainer approved for architecture planning; implementation not authorized.**

Authority: the maintainer's explicit “Base Architecture v1 Decision Consolidation” instruction accepts all D01–D11 recommendations **in principle** and selects P01–P11 as **approved policy**. This record documents that instruction; it does not turn earlier assistant recommendations into independent authority. [Current reading guide](CONSOLIDATION-AND-MILESTONES.md), [proposed B0 contracts](B0-ARCHITECTURE-CONTRACTS.md), [remaining specifications](SPECIFICATION-GAPS.md).

## Authority and scope

The approved principles/policies constrain B0 contract planning. The concrete K01–K11 specifications remain proposed for review; B0 completion and B1 execution are not declared. No implementation, migration, commit, push, PR, deployment, service ownership or accepted-ADR change is authorized. No numeric timeout, queue capacity, schema-version value, exact compositor environment or async executor is selected unless explicitly recorded below.

Accepted [ADR-0006](../../ADR/0006-renderer-selection.md) and native ownership/placement/compositor ADRs remain unchanged. Slint is the initial renderer, the concrete baseline is CPU software rendering with Core `wl_shm` submission, and one presenter serves each native generation. The initial foundation remains one Cargo package. Portability is a requirement, not support certification. Gate P remains unpassed; no Gate R/benchmark execution or promotion is authorized. The original S1–S6 scope, PARTIAL/UNVERIFIED/BLOCKED and historical NO-GOs remain intact.

The [original open-decision table](OPEN-DECISIONS.md) is preserved as the pre-approval wording. Its D01–D11 approval status is superseded by this record, not its source findings or unresolved engineering details. Original draft bytes and the old provenance manifest are bound in [new consolidation provenance](B0-CONSOLIDATION-SOURCES.json).

## D01–D11 — approved in principle

Each question below is reproduced from the original table without expanding its scope. In-principle acceptance does not select still-unspecified policy parameters or claim implementation.

| ID | Original question / scope | Status | Consolidated planning contract | Specific policy / remaining detail |
|---|---|---|---|---|
| D01 | Is Base Architecture v1 explicitly the single-package internal foundation, distinct from full Product v1? Which first compositor/version, single/multi-output, input, visual quality and CPU-rendering requirements are mandatory? | APPROVED IN PRINCIPLE | Single-package internal foundation, distinct from Product v1. B1 starts with one declared combination/output/scale 1; broader support is still evidence-dependent. | P01, P11; G01 |
| D02 | Approve one owner-thread App, neutral state/sizing and generation-bound event application, with Slint-only UI materialization/translation/painting behind a narrow boundary and concrete Core SHM submission? | APPROVED IN PRINCIPLE | One native/UI owner-thread App; SCTK owns native lifecycle and concrete SHM submission; a narrow concrete Slint bridge owns UI projection/translation/painting. Neutral values and relevant generations cross that boundary. | P01, P02; K01/K02/K04/K05 |
| D03 | Approve one host registry, one widget/controller registry, validated canonical attachments, separate UI incarnation and native token, with explicit per-host output context? What is the intended rehost/reset policy? | APPROVED IN PRINCIPLE | One canonical host registry, widget/controller registry and attachment authority with per-host output context. Logical identity/state survive rehosting and native recreation; UI/native incarnations may change. | P03; K03/K04; G03/G14 |
| D04 | Retain User-Open Focus after explicit clock activation; define non-text keys, repeat/scroll/capture/seat-loss and input regions. Choose what “ready” means and acceptable integer-buffer fractional quality. | APPROVED IN PRINCIPLE | Retain User-Open Focus after explicit clock activation. Specify input/geometry/quality per declared environment; keep three protocol milestones separate from visual acceptance. No universal focus grant or selected final readiness timeout/trigger follows. | P02, P11; K06/K07; G02/G05 |
| D05 | Approve optional provider modes, explicit demand, bounded messages, command outcomes, cancellation/completion and connection recovery? Which executor, if any, is necessary for the first provider? | APPROVED IN PRINCIPLE | Optional shared providers, explicit demand/modes, bounded messages, typed outcomes, cancellation/completion and recoverable connection ownership. No selected global executor or universal provider defaults. | P10; K01/K10; G12/G15 |
| D06 | Define desired hosts/widgets/attachments/service/theme settings; keep runtime placement separate; choose file/GUI/IPC validation parity and initial format/precedence. | APPROVED IN PRINCIPLE | Versioned TOML desired configuration and one validation/application path. Separate host policies, widget settings and attachments from runtime persistence; GUI and general IPC are not initial requirements. | P04, P05, P07, P08; K08; G06/G07/G08 |
| D07 | Approve action fence→cancel/complete→presenter/UI retirement→native destruction→persistence/flush→exit, including partial startup/repeated requests. Decide full Wayland loss: explicit stop/restart versus reconnect. | APPROVED IN PRINCIPLE | Ordered shutdown, partial-startup unwind and repeated-request handling. Complete Wayland loss uses explicit stop/restart; no in-process reconnect or approved automatic restart supervision. | P07, P09; K05/K11; G04/G13 |
| D08 | Choose config versus committed intent versus defaults, concurrent-writer/single-instance policy, durability guarantees and treatment of corrupt/unsupported versions. | APPROVED IN PRINCIPLE | Explicit config > permitted saved runtime intent > defaults. Single-writer persistence, protected last-valid state and visible unsaved/error outcomes; no automatic schema migration or future-version overwrite. Mechanism and durability specifications remain open. | P05–P08; K09; G09/G10/G11 |
| D09 | Keep Standalone and UWSM capability boundaries; approve authoritative manager detection and launch/notification contract only when needed. | APPROVED IN PRINCIPLE | Standalone-first, capability-truthful session behavior. Preserve session/UWSM boundaries, but real manager detection, managed launch and notifications remain deferred until separately specified/authorized. | K11; existing C07; session ADR candidate A06 |
| D10 | Is IPC needed in the first usable base? If so approve private session namespace, peer/authority, framing/size/version limits, command allowlist and same-owner socket cleanup. | APPROVED IN PRINCIPLE | General-purpose IPC is deferred. Future file/GUI/IPC clients share validation semantics; the single-writer contract is not permission to implement IPC. | P04, P06; G09; IPC ADR candidate A07 |
| D11 | Accept independently implemented Linux concepts now; require an explicit applicable project license/permission before direct GPUi Shell source/asset reuse? | APPROVED IN PRINCIPLE | Independently implement useful Linux-integration concepts. Direct GPUi Shell source/assets reuse still requires verified applicable licensing/permission and preserved notices. No code-copy approval is granted. | K10; preserved GPUI-LINUX-REFERENCE.md |

## P01–P11 — approved policy

The following text records the explicit maintainer policies. Derived engineering contracts may not weaken them.

### P01: B1 validation scope

**Status: APPROVED POLICY.**

B1 begins with one documented compositor/renderer combination, one output and scale 1.

This is an initial validation boundary, not a reduction of Product v1 goals or broader portability requirements.

### P02: Readiness

**Status: APPROVED POLICY.**

Separate configured, buffer-submitted and callback-observed protocol milestones from visual acceptance.

Do not equate protocol readiness with visible correctness.

### P03: Widget rehosting

**Status: APPROVED POLICY.**

Preserve logical widget identity and controller state through native surface recreation and rehosting.

UI incarnations and native tokens may be replaced.

State resets require explicit semantics, not incidental surface lifecycle effects.

### P04: Configuration format

**Status: APPROVED POLICY.**

Use TOML as the initial human-readable configuration format.

Use an explicitly versioned schema and a shared validation/application path.

GUI configuration and IPC are not required in the initial base.

### P05: Configuration precedence

**Status: APPROVED POLICY.**

Explicit configuration has priority over saved runtime intent, which has priority over defaults.

Saved intent may fill only permitted runtime values and must not override explicit configuration.

### P06: Writer authority

**Status: APPROVED POLICY.**

Use a single-writer contract for runtime persistence.

Prevent competing instances from writing the same state files.

This does not authorize general-purpose IPC.

### P07: Persistence failure

**Status: APPROVED POLICY.**

Preserve the last valid persisted file state.

Allow validated in-memory changes to remain active after a persistence failure, but report the unsaved/error condition explicitly.

Do not silently overwrite corrupt or unsupported state.

Distinguish application success from persistence success.

### P08: Schema migration

**Status: APPROVED POLICY.**

Do not implement automatic schema migration in Base v1.

Unknown future schema versions must not be overwritten.

Provide explicit diagnostics and a controlled fallback where safe.

### P09: Wayland connection loss

**Status: APPROVED POLICY.**

Use ordered shutdown and explicit stop/restart behavior for complete Wayland connection loss.

No in-process reconnect in Base v1.

Automatic restart supervision remains a separate future decision.

### P10: Service backpressure

**Status: APPROVED POLICY.**

Use bounded messaging, explicit cancellation/completion and typed command outcomes.

Coalesce replaceable state snapshots where appropriate.

Never silently discard significant commands or errors.

Do not select a global async executor or universal provider defaults prematurely.

### P11: Input and scaling quality

**Status: APPROVED POLICY.**

Define acceptance criteria for each declared validation environment.

Include applicable input-region, seat-loss and non-text input behavior.

Keep native fractional-buffer support and broad visual quality qualification as separate gates.

Do not infer broad compatibility from a single validated environment.

## Interpretation boundaries

- P02 selects separation of observations, not a single final readiness boolean, timeout or a presentation guarantee.
- P03 preserves logical state; it does not require retaining every UI/native object. Stale work is still rejected under the relevant lifetime contract.
- P05 selects desired-value precedence. Runtime topology fallback remains possible under ADR-0004 without rewriting explicit preference or saved home. It is not a higher-priority source of intent.
- Invalid configuration or failed application keeps the previous valid active state. A valid change successfully applied in memory but not saved may remain active under P07, marked unsaved. These are different failure classes.
- Unknown-future-schema fallback must not enable subsequent normal save or shutdown to overwrite that file. No migration or destructive repair is approved by P08.
- P06 requires a single writer for the protected runtime files, not a globally unique process, a chosen lock primitive, a general IPC server or a systemd dependency.
- On complete Wayland loss, ordered local cleanup cannot require successful acknowledgements or flush on an unusable transport. Report unavailable server-side confirmation honestly; no reconnect or automatic restart is approved.
- B1's one-combination/output/scale-1 scope is an initial acceptance boundary, not product qualification. Wider input/scaling, native fractional buffers, physical hotplug and portability need their own evidence.

This record selects no new renderer, crate split, plugin framework, service catalogue, background daemon or direct upstream source reuse. Proposed future ADRs are listed in the consolidation guide; none is adopted or numbered into the accepted ADR sequence here.
