# S5 — independent two-output functional parity, private SwayFX

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/results/2026-11-05-s5-multi-output/README.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

2026-11-05. This is a functional slice, **not a benchmark, renderer selection, production migration, Gate R or Gate P approval**. [Authorization](../../SOURCE-REFERENCES.md#source-d64510d25df6d679), [frozen scenario](../../SOURCE-REFERENCES.md#source-b034c88b1d298102), [independent review](../../reviews/2026-11-05-s5-multi-output.md).

**Slint03: scoped S5 PASS. egui02: scoped S5 PASS.** Both are supported by independent runtime/visual/cleanup review. Raw runtime gates retain their original PARTIAL pending-review labels; completed supplemental scoped acceptance is recorded separately. Raw files are immutable. Historical Slint01, Slint02 and egui01 FAILED/BLOCKED remain preserved separately.

## Ownership and implementation

Only experimental fixture/instrumentation changed. Production `src/`, `ui/`, dependencies/lockfiles, native GPU presentation/format-selection policy, compositor configuration, historical measurement artifacts and ADR-0006 are unchanged.

| Module | Concrete S5 change |
|---|---|
| `experiment/functional-parity/src/model.rs` | Fixed A/B IDs and independent actual Core clock/calendar controllers, registrations and queues; explicit shared64-intent process budget; old constructor preserved |
| `input.rs` | Calendar policy uses own model ID; both native leaves remain forwarded; idempotent cancellation regression |
| `slint_ui.rs` | Pair IDs and one shared RenderBridge through with_bridge; single platform installation; generation in observational input receipt |
| `egui.rs` | Model-derived host IDs; separate per-host Context and existing native presenter; no wayapp |
| `main.rs`, `s5.rs` | Concrete two-pair UiRuntime over the same WaylandPlatform/SurfaceManager/Coordinator/calloop; both queues drained, per-host repaint routing/deadlines, exact output preferences, snapshots and bounded controls; Slint global timers advance once |
| `validation/s5_multi.py`, `test_s5_multi.py` | Shared semantic sequence, connection/lifetime-aware protocol oracle, actual input and domain assertions, fixed visual geometry checks, deadline/idle/cleanup controls and negative regressions |
| `validation/output-observer/` | Default90s unchanged; explicit S5-only fixed240s whole-observer safety window and receipt. No arbitrary timeout; private guards and protocol behavior retained |

No new Core native lifecycle or renderer plugin interface. Core owns connection, output registry, roles, placement, generations and destruction. Slint uses exactly one Core software/wl_shm presenter per generation. egui uses one native wgpu presenter per generation, with no Core shm pool/buffer and GPU release before role/surface destruction. This remains an isolated fixture, not a production composition change.

## Source and reproduction

Base signed `352fdf978cd07206622a282ddc659d48c8f4c23e`.

- Rust fixture/source preservation: `a3310c4eac9cbf6c4a01b69ef31da095eb8c2400`.
- Initial software readiness correction and Slint01 evidence: `fdfd20f3178c53fba2d9e8ca451b3fb6033831d5`.
- Reviewed observer window/failure attribution and Slint02 evidence: `5a3f41b8a5ec99c585a016a666967140a70edb51`.
- Slint03 executed at `5a3f41b8a5ec99c585a016a666967140a70edb51`; egui01 used the same revision and stopped on the false deadline-oracle rejection. The reviewed helper correction is signed at `94b2adf83f3cc567ebe3abf6c642bc4133149871`, used by egui02. Slint03 A/B deadline evidence was independently replayed against the corrected helper.
- Both use the same Rust fixture binary, probe, compositor configuration, output/input sequence and functional criteria. The helper-only normalization is explicit; do not claim identical harness revisions. [Build hashes/settings](../../SOURCE-REFERENCES.md#source-073171c8fb3ac187), [observer build](../../SOURCE-REFERENCES.md#source-4bc67224677ed37d), implementation diff and per-run receipts bind actual files/binaries. No Rust/probe/compositor/config/binary change between these runs.

```sh
CARGO_HOME=<PRIVATE_PATH> \
CARGO_TARGET_DIR=<PRIVATE_PATH> \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 RUSTC_WRAPPER= \
cargo build --locked --offline --manifest-path experiment/functional-parity/Cargo.toml -j2
make -C experiment/functional-parity/validation/output-observer BUILD=/tmp/owlen-s5-functional-observer-build
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s experiment/functional-parity/validation -p 'test_*.py'
```

Exact executed Slint03 command (egui02 differs only in renderer and fresh output directory/driver log):

```sh
PYTHONDONTWRITEBYTECODE=1 python3 experiment/functional-parity/validation/s5_multi.py \
 --renderer slint --output /tmp/owlen-s5-multi-slint-03 \
 --binary <PRIVATE_PATH> \
 --sha256 ff9842b363453494bb85e31d3182d20092a684e576e15434a13d7e2a0c2dccb1 \
 --input /tmp/owlen-s5-presenter-input-build/s2-input \
 --input-sha256 d4d63bf76d7861a6dd3af942c8b1d4b8f886a640e9d776266fb04b7fdbcf2874 \
 --observer /tmp/owlen-s5-functional-observer-build/output-observer
```

The run must use a fresh owned private directory, explicit socket, peer-PID checks, isolated HOME/XDG state and tracked groups. Reproduction needs a suitable GPU/headless SwayFX environment; binary/source hashes and runtime dependencies may differ on another machine and must be recorded. These commands document authorized historical execution, not standing authorization for another run or benchmark.

## Evidence quality and failed attempts

- **Pre-execution NO-GO retained:** initial readiness oracle retried critical ValueErrors and reused older callbacks for incomplete new attachments/submissions. Independent disposable negatives falsified these cases. Corrected explicit pending-only retries and latest-own-callback requirements were independently tested before execution. Deadline paths require actual state, redraw, new submission, callback and visible indicator; fixed350ms idle uses existing S3 stable_window.
- **Slint01** [raw failed verdict](../../SOURCE-REFERENCES.md#source-4d11b94d78720dcd): initial unconfigured software surface legitimately had no pool/buffer; oracle called it wrong presenter. Reviewer traced existing Core first-paint allocation. Correction makes this bounded nonPASS readiness, still rejects competing native presenters and requires actual shm commit/callback. No code/presentation change; original failed run unchanged. Guardian cleanup passed, not graceful shutdown acceptance.
- **Slint02** [raw failed verdict](../../SOURCE-REFERENCES.md#source-be2d6a4afd75d49a): child-exit detection after approximately91s. Exact observer source/binary contains90s alarm; timeline strongly supports observer budget exhaustion. Original receipt lacks wait status, so exact process/signal remains UNVERIFIED; do not call this a Slint rendering failure. Complete earlier screenshots/domain/protocol evidence preserved; unfinished scenarios not passed. A fixed240s S5-only safety window was prospectively reviewed, preserving default90s and all functional barriers; it is common to both renderers. Future failure receipts add identity and non-reaping WNOWAIT status. Collector unchanged.

- **egui01** [raw failed verdict](../../SOURCE-REFERENCES.md#source-156bc04e7440b0d3): legitimate egui redraw-request can precede the indicator update during painting; Slint logs redraw-requested after its timer mutation. Raw own-arm→known redraw→indicator→new commit/submission→callback exists, but the oracle required Slint's internal order/name. A reviewed helper correction preserves the same functional requirements and validates source-supported ordering without extra repaint demands. Its actual deadline pixels and full normal teardown remain UNVERIFIED in that aborted trial; raw failure is unchanged.

No failed attempt was retried invisibly, relabeled passing or discarded. No optional GPU feature, alternate compositor renderer, format fallback, output-removal or production modification was used to force acceptance. Stop if the240s window is insufficient or a critical functional invariant fails.

## Environment and scaling scope

Same installed pinned SwayFX binary/configuration as capability and presentation prerequisites. Effective renderer is **SceneFX FX/OpenGL ES3.2 Mesa26.2.6/radeonsi/GBM on renderD128**, headless backend. WLR_RENDERER=pixman remains the historical configured preference; logs establish that it does not select the effective renderer here. No silent backend substitution.

HEADLESS-1 mode2560×1440@60000mHz, HEADLESS-2 mode1920×1080@60000mHz. Two distinct observer globals/lifetimes/proxies and Core output identities remain active; numeric IDs are connection-scoped. Initial positions0,0 and2560,0; independently move HEADLESS-2 to2200,120. Preferences swap A→HEADLESS-2, B→HEADLESS-1 after native recreation. [Initial IPC/registry](../../SOURCE-REFERENCES.md#source-15be960b692f790c), [final IPC/registry](../../SOURCE-REFERENCES.md#source-4171d720623bca5c).

| IPC output scale H1/H2 | Logical dimensions H1/H2 | Integer buffer scale H1/H2 |
|---|---|---|
| 1/1 | 2560×1440 /1920×1080 | 1/1 |
| 1/1.5 | 2560×1440 /1280×720 | 1/2 |
| 1.25/2 | 2048×1152 /960×540 | 2/2 |

Bars configure output logical width×32; physical buffers are those dimensions multiplied by integer scale. Calendars configure320×320 and buffers320×320 or640×640. SwayFX expected calendar top72logical includes bar exclusive32 plus margin40; right margin24. Captures remain physical mode size. This is **integer-buffer compatibility with compositor scaling**, not native fractional-buffer support. No physical hotplug, Niri multi-output compatibility, pixel/font/alpha-buffer equivalence, latency or utilization claim follows.

## Multi-output functional parity matrix — unranked

PASS means only the declared functional contract and cited evidence. [Structured facts](../../SOURCE-REFERENCES.md#source-9ae7db7c130657a0), [offline audit](../../SOURCE-REFERENCES.md#source-2734f612dfc94659), [cleanup audit](../../SOURCE-REFERENCES.md#source-953db08f873a29c8), [exact diff](../../SOURCE-REFERENCES.md#source-545afc372c2b8eb4). Complete runtime source revisions and binaries above remain distinct from the final documentation commit.

| Contract | Slint03 | egui02 | Reproducible evidence, relative to the candidate run directory |
|---|---|---|---|
| Two active independent output identities/configuration | PASS | PASS | `initial-outputs.json`, `mixed-1-1p5-outputs.json`, `mixed-1p25-2-outputs.json`, `position-b-outputs.json`, `final-outputs.json`: IPC+independent registry correlation, unchanged distinct globals/lifetimes/proxies |
| Simultaneous four native hosts, two distinct initial controller months | PASS | PASS | `opened-b-state.json`, `opened-b-wire.json`, `opened-b-a.png`, `opened-b-b.png`: A November/B December, actual full views and separate tokens/presenters |
| Output-specific pointer enter/press/release without preceding motion | PASS | PASS | `checks.json`, per-click coordinates/input logs and raw client protocol/Core/UI evidence;15 clicks each have one intended accepted action |
| Calendar navigation and cross-output state isolation | PASS | PASS | `navigated-a-state.json`: A January2027/B December2026; `navigated-b-state.json`: A January2027/B November2026; `closed-a-state.json`, `reopened-a-state.json`, screenshots |
| Independent hide/reopen and native-generation replacement | PASS | PASS | `hidden-deadline-a-state.json`, `reopened-after-deadline-a-state.json`, `recreated-a-state.json`: B unchanged; A intended month retained, newer generations |
| Exact output preference swap and placement | PASS | PASS | `swapped-state.json`, `swapped-wire.json`, `swapped-a.png`, `swapped-b.png`: A→HEADLESS-2, B→HEADLESS-1, newer tokens and unchanged controllers |
| Mixed-scale logical/configured/buffer geometry and hit testing | PASS | PASS | `mixed-1-1p5*`, `mixed-1p25-2*` output/state/wire/capture/coordinate files; both close/reopen sequences, scale classification above |
| Independent output position change, visibility and input | PASS | PASS | `position-b-outputs.json`, `position-b-state.json`, `position-b-reopened-a-state.json`, corresponding full images and coordinates |
| Per-output deadline expiry→state/redraw→new submission→callback | PASS | PASS | `a-deadline-fired-deadline.json`, `b-deadline-fired-deadline.json`, state/wire JSON and complete captures with visible magenta indicator; framework-specific causal ordering, not equal event counts |
| Retirement cancels old deadline and queued old UI intent | PASS | PASS | `a-deadline-cancellation.json`, `stale-control-a-state.json`, raw `action-rejected-stale` and `s5-retired-control`; one rejection, no extra accepted action |
| Deterministic stale native-token rejection | PASS (offline Core regressions) | PASS (offline Core regressions) | [Core test log](../../SOURCE-REFERENCES.md#source-eb488dbc7f5a8d6f), fixture tests; actual native replacement separate. **Controlled late native-callback runtime timing UNVERIFIED**, not invented by a synthetic event |
| Functional quiescence after interactions | PASS,350ms only | PASS,350ms only | `functional-idle.json`, raw bounded event interval ends before caller's explicit quit wake; no utilization/efficiency or long-idle claim |
| One presenter per native generation | PASS: Core shm | PASS: native wgpu, no Core shm | All46 accepted/settled state-wire associations per renderer; actual buffer type/geometry and callback evidence |
| Full normal teardown/resource ordering | PASS,17 lifetimes | PASS,17 GPU-before-native lifetimes | `teardown.json`, raw role/surface destroys; client/observer exit0, `shutdown.failed=false`, native instances0 |
| Owned process/group/session cleanup | PASS,20 owned processes | PASS,20 owned processes | `cleanup.json`, result/attribution, [post-run ownership audit](../../SOURCE-REFERENCES.md#source-953db08f873a29c8), empty runtime |
| Native leave uniqueness / semantic cancellation | PARTIAL raw duplicate pattern; PASS idempotent effects | PARTIAL raw duplicate pattern; PASS idempotent effects | Each trace has28 raw protocol leaves:22 surface-bound and6 nil after destruction;22 Core/UI PointerExited events split barA12/barB6/calendarA2/calendarB2. All retained; held state cleared; no extra domain action. Mechanism not newly resolved |
| Keyboard/focus | UNVERIFIED | UNVERIFIED | Pointer-only capabilities0/1, no keyboard exercises. No denied-focus or universal Niri/Sway policy claim |
| Shared scheduled clock ticks in this S5 composition | UNVERIFIED | UNVERIFIED | Frozen12:34:56 fixture; accepted S3 historical evidence stays separate |

Run directories: [Slint03](../../SOURCE-REFERENCES.md#source-dc2db4a593fa889f), [egui02](../../SOURCE-REFERENCES.md#source-6ff1ef80c04d43c4). Each retains38 complete output screenshots plus raw client/compositor/observer logs, state/geometry/coordinates, buffer/callback and teardown evidence. The independent reviewer inspected16 complete representative scenes per renderer across simultaneous display, navigation, swap, deadline indicators, both scale phases and independent position change. Automated geometry/content checks cover every capture. This is not maintainer human visual approval or pixel/font/alpha equivalence.

## Tests, runtime metadata and preservation

- **132 Core/integration tests PASS**; **40 library+8 fixture-main tests PASS**; **84 Python harness tests PASS** after all scoped corrections. Two additional existing observer-refusal tests were repeated against the rebuilt observer: PASS. Logs retain initial compilation errors, initial harness NO-GO, every failed runtime and review remediation. Rust formatting passed; no new Rust change after the tested build.
- Slint: actual software paint and wl_shm buffer submissions. egui: all17 [format/adapter receipts](../../SOURCE-REFERENCES.md#source-25e961cfaeb5dae0) select advertised Bgra8Unorm with device features `[0,0]`, Vulkan/RADV RX9070XT, Mesa26.2.6. Provisional initialization dimensions in selection metadata are **not applied buffer dimensions**; actual geometry comes from configure/ack, gpu-configured/Core submission and wire buffers.
- [Environment inventory](../../SOURCE-REFERENCES.md#source-03805e88f82eb830), per-run environment/attribution, exact command and pinned lockfiles identify the toolchain, GPU/compositor and configuration. No old exploratory measurement is reattributed to these later launches.
- `raw-identities.json` in every run binds original decoded bytes and hashes; logs are lossless gzip with mtime0. All1196 raw original artifacts from five attempts are retained. The artifact manifest binds the final report set. No binaries, generated protocol files, shader/Cargo caches or private runtime sockets are committed.
- `audit.py` verifies raw preservation, historical failed verdicts, all92 normal-run state/wire associations, canonical domain transitions, ten registry/IPC checkpoints, both deadline paths and separate latest callbacks, bounded idle, actual presenters/teardown and runtime metadata. Run it offline with `PYTHONDONTWRITEBYTECODE=1 python3 docs/bakeoffs/renderer/results/2026-11-05-s5-multi-output/audit.py`; it launches no client/compositor or measurements. Supplemental visual review is separate.

## Known scope limits

Keyboard/focus UNVERIFIED without actual keyboard capability/exercises; no Niri focus policy inference. Shared domain-clock ticks are unexercised in S5 (frozen fixture clock), not newly passed from S3 historical results. Existing Core regression tests establish deterministic stale native-token rejection; S5 adds real generation replacement and stale UI-incarnation rejection, without claiming a new controlled delayed-native-callback race. Device recovery, physical output lifecycle/S6, prolonged soak and production acceptance remain outside scope.

Historical Niri multi-output BLOCKED and all historical NO-GOs remain unchanged. ADR-0006 remains Proposed; private Niri diagnostic disabled. COSMIC remains WITHDRAWN — NOT FULLY EVALUATED. No other renderer candidates executed.

Renderer selection remains a maintainer decision. No winner has been selected.
