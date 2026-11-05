# S6 — output lifecycle and graceful shutdown, private SwayFX

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/results/2026-11-05-s6-lifecycle/README.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

2026-11-05. [Authorization](../../SOURCE-REFERENCES.md#source-b53b80bf08ed75e8), [prospective contract](../../SOURCE-REFERENCES.md#source-d58f4b4065215b5c), [independent review](../../reviews/2026-11-05-s6-lifecycle.md). Functional validation only: no performance measurement, production integration, renderer ranking or Gate R/P promotion. ADR-0006 stays Proposed; the private Niri diagnostic stays disabled.

Slint: **7/7 scoped scenario PASS**. Native egui: **7/7 scoped scenario PASS**. Overall S6 coverage remains **PARTIAL**, with keyboard-focused removal, controlled native wire races and physical compatibility explicitly unverified. These are scoped results for the declared synthetic SwayFX scenarios, with the UNVERIFIED items below retained. They are not complete production acceptance.

## Implementation and ownership

The original OwlenShell checkout is unchanged. Core changes below exist only on the dedicated experimental branch.

| Affected module | Narrow change |
|---|---|
| `src/platform/surface_manager.rs` | Remove native routes and invalidate the generation before input cancellation, presenter release and native destruction; latest-allocated identity check |
| `src/platform/wayland.rs` | Closed carries its original output identity; stale frame callback rejection trace |
| `src/runtime/coordinator.rs` | Validate frame/close host and generation; distinguish original-output loss from live-output close policy |
| `experiment/functional-parity/src/model.rs` | Idempotent shutdown fence, pending/new-action rejection, UI registration retirement/counting |
| `slint_ui.rs`, `egui.rs` | Actual per-host UI removal/rematerialization, fresh UI incarnations, retained Core controllers; fix Slint recreated-bar weak-reference coupling |
| `main.rs`, `s6.rs` | Existing calloop/Coordinator composition with a missing-output retirement fence, private signalfd, private placement persistence and bounded scenario controls |
| `validation/s6_environment.py`, `s6_lifecycle.py` and focused tests | Real registry removal/rebirth, strict state/wire/resource/process oracles and bounded scenario execution |

Core retains the one SCTK connection/dispatch, output tracking, native layer roles, generations, placement, persistence and input routing. Exactly one concrete presenter remains active per native token: Slint Core software/wl_shm, or native egui wgpu/dmabuf with no Core shm presenter. No wayapp, general plugin framework, event-loop replacement or GPU-feature/format-policy change.

Actual output loss retires old native resources and UI registrations before rematerializing fallback UI, retaining calendar/clock controllers and visibility intent. Native-only restoration from a still-live fallback output creates a fresh native token while retaining valid UI registration, as FP1 permits. Original preferred output names are never rewritten. Zero outputs leave logical visibility intact while all native instances and UI registrations are absent.

SIGINT/SIGTERM are blocked before UI/GPU threads and consumed through private signalfd. Shutdown fences actions, cancels scheduling, saves placement intent only, retires native epochs/presenters/UI, joins owned repaint threads, drains already-queued late work without remapping, flushes and returns. Signal masks stay blocked through return; duplicate standard signals can coalesce. An application exit and a harness cleanup receipt are separate observations.

## Source and reproduction

Base signed `392f70b0756845969fd7c4715f29fc4a3890c269`. Environment-only source `6f7c1a0586601abbbf332967d8dc01949332bffa`; Rust implementation `fbcf761e174164d809b9bd863fc9aed63e939516`. [Build settings/source hashes](../../SOURCE-REFERENCES.md#source-82f7913e99a36e25): one unchanged binary `1518f2540a9b33fc62fa2da64a982b8f9b4db96eb2a6b61b393f693ec850793d`, private offline target, no experimental dependency change.

Slint lifecycle04 and six remaining cases executed with reviewed driver at `c8cc41318c440573cbc4bf608e0dce45cc56da6d` (hash `6a7c402a948bf6a1abe07d02e32dd2dbe757a49e2c8b74f707e949c77fc315d0`). egui lifecycle02 and remaining cases use the stricter callback-accounting correction `5be9f6119a1e9e19c9c525f7a660f89c1e097837` (hash `94ebdd5d313afa408146ba217337b7504d93ba2b0b8a6029c9b23d126cafe51d`). No Rust binary, compositor configuration or functional criterion changed between them. Different harness revisions are explicit; no workload/performance equivalence is inferred. Original failed runs retain their own source identities.

```sh
CARGO_HOME=<PRIVATE_PATH> \
CARGO_TARGET_DIR=<PRIVATE_PATH> \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 RUSTC_WRAPPER= \
cargo build --locked --offline --manifest-path experiment/functional-parity/Cargo.toml -j2

PYTHONDONTWRITEBYTECODE=1 python3 experiment/functional-parity/validation/s6_lifecycle.py \
 --renderer egui --case lifecycle --output /tmp/owlen-s6-egui-lifecycle-02 \
 --binary <PRIVATE_PATH> \
 --sha256 1518f2540a9b33fc62fa2da64a982b8f9b4db96eb2a6b61b393f693ec850793d \
 --input /tmp/owlen-s5-presenter-input-build/s2-input \
 --input-sha256 d4d63bf76d7861a6dd3af942c8b1d4b8f886a640e9d776266fb04b7fdbcf2874 \
 --observer /tmp/owlen-s5-functional-observer-build/output-observer
```

A reproduction requires a fresh owned direct `/tmp` directory and appropriately rebuilt/pinned binaries; never overwrite a prior run. Case names are lifecycle, restart-pending, sigterm, sigint, repeat, fault and abrupt. Restart additionally requires `--placements-input /tmp/<prior-run>/state/placements.toml --placements-sha256 <exact-receipt-hash>`; it copies only private logical placement intent. Twelve worker-launched case command receipts and all nineteen external driver logs are preserved and directly hash-bound in each archive. Parent-launched lifecycle/environment sessions did not emit contemporaneous command.json receipts; their exact tool invocations are preserved as explicitly labeled [post-execution reproduction records](../../SOURCE-REFERENCES.md#source-dd969fd8b7bf722d), not fabricated as runtime receipts. Environment and identity receipts record actual execution. These are historical reproduction instructions, not standing authorization for further runs or measurements.

Effective compositor is pinned private headless SwayFX 0.6/Sway1.12.0, SceneFX FX/OpenGL ES3.2/Mesa26.2.6/radeonsi/GBM. The inherited `WLR_RENDERER=pixman` preference does not describe the observed renderer. Initial HEADLESS-1 is2560×1440 at0,0, HEADLESS-2 is1920×1080 at2560,0, scale1. Actual IPC, independent registry observer and candidate connection identities are recorded for every transition. No production compositor socket, host monitor setting or desktop service is used. Niri multi-output remains historically BLOCKED.

## Functional acceptance and evidence

| Scenario | Slint | Native egui | Verified contract |
|---|---|---|---|
| lifecycle | [PASS](../../SOURCE-REFERENCES.md#source-07265fb087e37f2b) | [PASS](../../SOURCE-REFERENCES.md#source-5e8e1c4a7496af6d) | Three A cycles, zero-output recovery, pending deadlines, explicit quit;27 native lifetimes each |
| restart-pending | [PASS](../../SOURCE-REFERENCES.md#source-56c7ddca269ffffb) | [PASS](../../SOURCE-REFERENCES.md#source-09a4f86102ee82c2) | Private placements loaded, calendars open and pending work, natural exit0 |
| sigterm | [PASS](../../SOURCE-REFERENCES.md#source-fcf4dc97fa273f22) | [PASS](../../SOURCE-REFERENCES.md#source-c9dd832459327645) | Direct pidfd SIGTERM reaches application signalfd; natural exit0 |
| sigint | [PASS](../../SOURCE-REFERENCES.md#source-4897a6ada911b124) | [PASS](../../SOURCE-REFERENCES.md#source-2983f32dafb59f42) | Direct pidfd SIGINT reaches application signalfd; natural exit0 |
| repeat | [PASS](../../SOURCE-REFERENCES.md#source-507f3fc5efb0e013) | [PASS](../../SOURCE-REFERENCES.md#source-ed080d3a96dfb013) | Exactly two explicit shutdown requests; no duplicate work; natural exit0 |
| fault | [PASS](../../SOURCE-REFERENCES.md#source-2565be6df2c14c2e) | [PASS](../../SOURCE-REFERENCES.md#source-b6434f1d82bca169) | Expected exit1/non-graceful: Slint first-paint failure; egui unbound after-device init failure |
| abrupt | [PASS](../../SOURCE-REFERENCES.md#source-98ff4df7629debb3) | [PASS](../../SOURCE-REFERENCES.md#source-7cd7c8837556fb17) | Expected SIGKILL/non-graceful; both complete compositor captures equal baseline |

[Structured facts](../../SOURCE-REFERENCES.md#source-6e7b16093eb099a4): five graceful application sessions and45 owned processes per renderer, all absent after cleanup. Each renderer has46 ordered native lifetimes across normal/fault cases. egui has45 successful GPU epochs plus one narrowly classified unbound-init failure. Abrupt cases supply no client destructor proof. Ten graceful cycles, two expected fault controls and two abrupt controls are distinct; none is a performance sample.

Each lifecycle run has three actual A registry removal/rebirth cycles, globals58→59→60→61, while Bglobal57 and its Core identity remain live. Both-output absence then creates new Bglobal62 and Aglobal63 and fresh Core identities. These are real protocol output lifetimes under the same connector names, not proof of physical connector hotplug or complete underlying headless wlr_output destruction. Inactive IPC rows are retained separately from live protocol outputs.

Both calendars start from actual clock clicks, navigate independently to A January2027 and B November2026, and retain those dates across fallback, restoration and total absence. Both fallback bar clocks remain reachable; observed order is B row16/A row48 with two exclusive32 zones and visible boundaries29/61. Popup overlap is permitted; simultaneous complete popup visibility on one output is not claimed. Restored captures show complete calendars on separate outputs, including titles, close controls and date rows. No pixel/font/alpha-buffer equivalence claim follows.

Real held-pointer removal causes no accepted domain action. Eight intended toggle actions and one explicit old-UI payload rejection occur per complete lifecycle run. The held stale payload is a fixture UI intent, not fabricated native input. Fresh exact-generation deadlines are armed before removal, cancel at retirement and cannot fire/submit afterward. Four fresh unexpired deadlines and redraw requests precede each normal shutdown; direct signal cases record actual application signal reasons. Repeated shutdown verifies two explicit control requests, separately from the terminal Coordinator request.

Every normal/fault native lifetime is checked for generation invalidation → input cancellation → presenter release → role destruction → native destruction. egui successful GPU lifetimes require GPU release before the native role/surface; the exact injected unbound after-device failure has a separately reviewed cleanup witness. Client API/resource release and protocol destruction are observed, not GPU-fence completion or driver allocation reclamation. Actual role-closed events are reconciled as output-lost in the tested ordering, with no live-output close-policy suppression. Closed arriving before global removal in another dispatch batch/compositor remains a portability risk.

Frame acceptance requires current configure/ack, matching buffer geometry/type, actual submission and completed Core callback. The revised egui oracle records queue identity, excludes Mesa auxiliary/discarded callbacks from acceptance, and permits canceled-ID reuse only after owning-surface destruction plus deletion acknowledgement. It supplies no completion credit to cancellation. This relies on the pinned logger's untagged Core versus named auxiliary queue conventions; it is not a general Wayland capture proxy. RGB compositor captures establish visible composition only, not actual alpha-buffer values, display timing or input-to-photon latency.

Independent supplemental review verifies the scheduled old egui Context callback: calendar-A generation1 retires at raw lines9474/9588, generation2 exists at9640, and generation1/pass16/ThreadId(2) is delivered13729 and rejected13730. The original late-real-egui-repaint.json UNVERIFIED label is preserved; this separate scoped finding proves that delivered renderer-wake rejection, not a precisely controlled native-wire race.

## Failed attempts and review

Four original failed runs remain immutable:

| Run | Finding | Reviewed correction |
|---|---|---|
| Slint lifecycle01 | Hidden survivor calendar had native=null; helper indexed it | Accept both absent, reject asymmetric absence; live identities remain exact |
| Slint lifecycle02 | Helper incorrectly required fresh UI on native-only preferred-output return | Preserve FP1: fresh native + same UI on return; actual output loss still requires fresh UI |
| Slint lifecycle03 | UI-local indicator false→None on retirement mistaken for Core state mutation | Compare Core state always, indicator invariance within same live UI; require absentUI→None/liveUI→bool |
| egui lifecycle01 | Canceled Mesa callback ID reused after surface destruction/delete_id, but parser retained it | Owner/deletion lifetime proof plus real same-queue Core completion; no blanket delete_id credit |

All stopped immediately, preserved raw logs/captures, completed owned cleanup and received independent root-cause/remediation review before retry. They remain FAILED/BLOCKED and do not become passing runs through offline reanalysis. No renderer application correction was needed during these retries. Original callback failure marker remains PENDING under the corrected parser, not retroactively PASS. Offline [callback analysis](../../SOURCE-REFERENCES.md#source-19810a4f6f8188d4) is explicitly callback regression evidence; its snapshot-derived IPC does not independently verify geometry. A separate final Slint replay uses actual saved IPC.

Initial review also caught stale output-proxy attribution, missing deadline-arm barriers, post-fence work/repeat-count false positives, existing-directory cleanup, neutral clock-color assumptions and nonzero-output-origin input normalization. Focused negatives cover attribution, shutdown, root, color and coordinate corrections; inline deadline-arm barriers have source review and actual arm/cancellation receipts, without a dedicated missing-arm unit test. A Rust late-dispatch error-report omission was fixed before candidates ran.

Tests:134 Core/integration tests,54 fixture tests and113 final Python harness regressions PASS. Independent reviewer separately reran focused tests and replayed raw acceptance/teardown controls. One early Python attempt failed because the sandbox denied a disposable Unix socket; its failed log remains preserved, and permitted socket tests subsequently passed. Intermediate discovery counts are not the canonical final suite. Full build/test logs are under tests/. Raw logs are compressed losslessly with original/stored hashes in each archive.json; sources, images, structured receipts and negative findings are retained. No binaries, shader caches or oversized traces are committed.

## Remaining gaps and S1–S6 status

| Requirement or slice | Slint | Native egui | Scope/evidence |
|---|---|---|---|
| S1 Coordinator bar/clock/calendar | Prior scoped PASS | Prior scoped PASS | [S1](../2026-11-04-s1-functional-parity/README.md); not a new full regression execution |
| S2 User-Open Focus | Prior87PASS/4UNVERIFIED | Prior87PASS/4UNVERIFIED | [Adopted S2](../2026-11-05-s2-user-open-focus/README.md), pinned Niri; original NO-GO preserved |
| S3 scheduling/functional idle | Prior scoped PASS | Prior scoped PASS | [S3](../2026-11-05-s3-scheduling/README.md); no utilization claim |
| S4A scales1/1.25/1.5/1.75/2 | Prior scoped PASS | Prior scoped PASS | [S4A](../2026-11-05-s4-scaling/README.md), integer-buffer compatibility; native fractional remains unimplemented |
| S5 simultaneous independent pairs | Prior scoped SwayFX PASS | Prior scoped SwayFX PASS | [S5](../2026-11-05-s5-multi-output/README.md); Niri BLOCKED unchanged |
| S6 declared lifecycle/shutdown scenarios | 7 scoped PASS; overall coverage PARTIAL | 7 scoped PASS; overall coverage PARTIAL | Current report, per-case raw evidence and independent review |
| Keyboard-focused removal / focus restoration | UNVERIFIED | UNVERIFIED | No keyboard capability exercised in private SwayFX; no inference from Niri User-Open Focus |
| Precisely forced native-wire callback race | UNVERIFIED runtime | UNVERIFIED runtime | Deterministic Core regressions and actual lifecycle separate from timing-controlled native race |
| Exact expiry/shutdown boundary race | UNVERIFIED | UNVERIFIED | Pending timers verified; no claim of exhaustive boundary interleavings |
| Occluded-popup pointer stacking / focus restoration | UNVERIFIED | UNVERIFIED | Reachable bars and separate-output popups tested; complete overlapping-popup interaction matrix not established |
| Physical connector hotplug / GPU device loss | UNVERIFIED | UNVERIFIED | Synthetic output-global lifecycle and partial-init errors do not prove physical/device recovery |
| GPU fences / driver memory reclamation | NOT APPLICABLE to software presentation | UNVERIFIED | API teardown observed; no GPU completion tracing or memory sampling |

GTK4, Qt6/QML and Iced received no S1–S6 execution. COSMIC/libcosmic stays WITHDRAWN — NOT FULLY EVALUATED. Historical Gate R NO-GOs, all failed attempts and the original exploratory non-decision-grade data remain unchanged. No performance conclusions, weights, rankings, renderer choice, production migration or additional execution authorization follow.

Renderer and architecture selection remain exclusively with maintainers. Stop after this S6 handoff and await separate approval.
