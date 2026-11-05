# B0 closure proposal — retirement, recreation and shutdown

2026-11-05. **Proposed specification; no implementation or runtime acceptance.** Refines K05/K11 and G04 under [MDR-0001](MAINTAINER-DECISIONS.md), P03/P07–P10. Read with the [generation/admission contract](B0-GENERATION-TOKEN-CONTRACT.md). This defines ordering now; phase deadlines and service/storage mechanisms remain G13/G09–G12 prerequisites at their milestones.

## Baseline and ownership

[SurfaceManager::retire](../../../src/platform/surface_manager.rs#L380) currently removes routes, drops the instance, then retires the registry token despite its contrary comment. The new order below is **not implemented**. SCTK's existing role-before-owned-surface destruction remains reusable; it is not a newly found defect. [Attach/recreate](../../../src/platform/surface_manager.rs#L408), [Slint creation rollback](../../../src/platform/render_bridge.rs#L172), [main](../../../src/main.rs) and [repository save](../../../src/persistence/repository.rs#L185) provide the concrete starting paths. Main's error exits do not establish complete orderly shutdown; temp-write/rename does not establish locking or durable publication. C02/C10/C13/C16/C19 remain OPEN.

Core remains the only native lifecycle and software/SHM submission owner. The Slint boundary owns toolkit state/CPU painting, not native destruction. There is exactly one presenter for a generation, including during failure/replacement. No speculative GPU bridge is introduced; any future GPU presenter must likewise release its surface-dependent resources before native destruction.

## Ordered per-generation protocol

All phases run on the owner thread. Record token, host registration, output lifetime, operation/reason and phase outcome. Repeated requests join the same retirement; they do not allocate another destruction transaction. Observable local release and server processing are different claims.

| Phase | Required action and completion criterion |
|---|---|
| R1 — fence | Mark the exact generation Retiring and revoke its current actionable token before any resource drop. Stop native-derived action admission, painting/submission and new callback/deadline requests for it. Remove native routes without invoking toolkit/user callbacks. Apply a live compositor-close policy once before replacement, as specified in the generation contract; later Closed receipts are cleanup-only. |
| R2 — invalidate/cancel | Invalidate generation-bound queue/deadline/frame authority. Clear only matching focus, pressed/capture/drag state using non-activating cancellation. Preserve identifiable late entries for sink rejection or remove them with recorded cancellation; never retarget them by HostId. Settle already-admitted logical commands according to the operation below. No synchronous wait for an incoming frame/release callback is required. |
| R3 — quiesce UI/presenter | Detach the native binding from the retained UI, or revoke the UI incarnation if rematerializing/removing it. Reject old painting/input requests; cancel toolkit work that belongs only to this binding. Mark the presenter inactive and release surface-dependent paint/SHM ownership before native destruction. No replacement presenter is admitted yet. |
| R4 — native destruction | Destroy layer role before its owned native surface, then remove remaining instance/route/registration records. Keep identity counters and needed terminal receipt records. Correct SCTK drop semantics can perform the role/surface step; do not create competing manual destruction paths. |
| R5 — completion | Record local resources released versus outstanding backing-buffer references versus unobservable server completion. Retained logical host/controller state is unchanged unless an explicit domain operation changed it. Replacement may start only once the old presenter/native binding is locally retired. |

**SHM release qualification:** a submitted buffer's backing storage must not be overwritten/reused while the compositor may still read it. Surface retirement stops submissions but is not a `wl_buffer.release` acknowledgement. Retain the required allocation/slot ownership independently of the dead surface until protocol release or connection teardown makes disposal safe. A release event may clean its own retired allocation without restoring action authority. Local destruction of proxies is not proof the server released references. Do not block owner-thread retirement waiting indefinitely for server release or an extra frame callback.

## Operation-specific ordering and retained state

| Operation | Exact extension of R1–R5 and result |
|---|---|
| Individual hide / native retirement | Set logical visibility if this is an explicit hide action, then R1–R5. Preserve host/widget registrations, attachments and controller/month state. UI may stay registered but inert, without a native binding. Independently admitted domain commands remain valid under their logical target checks; hiding is not logical deletion. No save is implied. |
| Native recreation / output fallback | R1–R5, preserve logical state and optionally UI incarnation, allocate a fresh native token, bind actual OutputId and current UI incarnation, install routes, configure/ack as required, then admit software painting/submission. Reset protocol-readiness ledger for the new token only. If creation fails, retire only that failed attempt and report an unrealized host; never revive the old token. Desired home/explicit config is unchanged by fallback. |
| Widget rehosting | Fence the old attachment/UI-origin authority first; settle already-admitted commands under the generation contract. Retain WidgetId/registration/controller and its data. Retire affected native/UI binding with R1–R5 if needed; invalidate the old attachment revision, validate the proposed new composition, establish fresh attachment authority and materialize/rebind the compiled UI projection. On failure keep the logical controller/state, restore a valid composition only via a fresh binding/revision or report unrealized state. Never resurrect old callbacks. No arbitrary dynamic Slint nesting is required. |
| Final logical removal | Fence that host/widget registration as well as native work. Explicitly cancel pending logical commands targeted at the removed registration; record applied commands as applied. Retire affected UI/native resources via R1–R5. Remove logical state only because the domain operation explicitly requested removal; retain allocation history for same-name re-registration safety. |
| Partial startup failure | Fence only the failed transaction; revoke its pending creation scope/routes/token and matching UI registration. Unwind acquired resources in reverse dependency order, presenter before native role/surface. Do not remove an unrelated registered adapter or existing controller. Failed identities remain consumed. Report original error and cleanup error separately; if resource ownership cannot be proven, stop further creation and fail the slice rather than retrying blindly. |

A complete rehost transaction is a later composition feature; B1 needs deterministic registry/controller/attachment tests and concrete native recreation only. This contract does not authorize arbitrary runtime composition or output-lifecycle implementation in B1.

## Complete application shutdown

The owner-thread App owns the following global sequence; B2 implements and validates it after separate authorization.

1. Latch the first shutdown reason; subsequent requests are idempotent and may be recorded. Fence **all** new logical/native/service command admission and stop creation/reconciliation. Do not invoke user activation callbacks during cleanup.
2. Cancel or complete outstanding admitted work with explicit outcomes. Already-applied domain transitions remain applied. Stop producers; request cancellation and observe completion of owned work without blocking the owner thread on work that needs its queue. In Base v1, no unsolicited new worker or executor is introduced. B5 must define bounds for its concrete workers.
3. Freeze the final canonical state after state-mutating work is settled/canceled. Invalidate generation-bound deadlines and queued effects, clear matching input state, and perform R1–R5 for every surface. Revoke remaining UI registrations and release toolkit/native bindings. Never count process exit as proof these phases ran.
4. Request final persistence **only if** there is an existing permitted dirty runtime intent, writer authority, supported/unprotected file and an approved storage path. Keep writer authority through the final permitted write. Do not create intent from topology fallback, migrate schemas, overwrite corrupt/future-version files or invent a save on every hide. Capture apply, publication, durability and unsaved/error outcomes separately; P07 permits valid in-memory state after save failure, not a false success. G09–G11 define storage guarantees before real-state use.
5. On a still-usable Wayland transport, attempt the approved bounded flush of already-issued destruction requests; a flush is not server processing or display confirmation. Release connection/event-source/remaining local buffer ownership, then repository guards and runtime ownership records. Finish session stop hooks where applicable without treating existing no-op hooks as real UWSM integration. Exit naturally with a truthful terminal outcome.

Failure handling: resource-release errors must not restore action authority. Continue independent safe cleanup, report incomplete phases and do not label forced harness termination graceful. G13 must specify concrete bounds, escalation/status and completion observations before B2 execution; no numeric limits or universal guarantee against uninterruptible kernel I/O is invented here.

## Complete Wayland connection loss

Latch the lost-transport reason and use the same global fence/settle/local-release/state-protection sequence. Invalidate every native generation and route; cancel pending callback/readiness waits. Release local presentation resources before native proxy/connection ownership. Do **not** require configure acknowledgement, buffer release, destruction delivery or flush on a dead connection; record server completion as unavailable. Safe connection teardown terminates local backing-buffer ownership without claiming server receipt. Persistence is allowed only under the same independent authority/protection rules and must not delay cleanup indefinitely. Stop with explicit restart-required diagnostics; no in-process reconnect, new Wayland connection, automatic relaunch or supervisor configuration is authorized by P09.

## Proposed tests / evidence, not new PASS results

| Control | Observable acceptance | Milestone |
|---|---|---|
| Ordered retirement spy, repeated retirement | Fence/token revocation precedes presenter release; role precedes native surface; one local completion; late work cannot mutate replacement | B1 |
| SHM release delayed beyond native retirement | No backing-store reuse before safe release; old release cleans only its allocation; no synchronous callback wait | B1 unit/model; actual wire timing remains separately scoped |
| Native recreate with queued input/frame/terminal receipt | Fresh token, retained controller/UI policy, no competing presenter or replacement effect from old work | B1 |
| Partial failure at each acquired UI/native/presenter stage | Matching-only reverse unwind, no retained presenter/routes/pending creation; retry consumes fresh identity | B1 |
| Model rehost and final logical delete/re-add | State retained on rehost; explicit delete cancels targeted work; old attachment/registration rejected | B1 model, live composition later |
| Shutdown with open calendar, pending work and repeated requests | Global fence, explicit outcomes, ordered local destruction and natural exit, no owned survivors | B2 private runtime |
| Full connection loss at initialization/active/retiring phases | No reconnect/flush wait; local cleanup ordered; protected state not overwritten; unavailable server completion recorded | B2 |
| Write failure / future-schema fallback followed by shutdown | Last valid recoverable state and protected bytes preserved; truthful unsaved/publication/durability outcomes | B4 with private fault fixtures |
| Worker cancellation/failure during shutdown | No silent command/error loss; bounded completion/status; no owner-thread deadlock | B5 |

Controlled native-wire races, physical output hotplug and keyboard-focused output removal remain UNVERIFIED. Historical S6 experiments do not prove that current canonical code already implements this contract.
