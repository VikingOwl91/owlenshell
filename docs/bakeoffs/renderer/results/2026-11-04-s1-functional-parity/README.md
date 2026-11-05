# S1: Coordinator-owned Slint/egui functional parity

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/results/2026-11-04-s1-functional-parity/README.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Runs started2026-11-04; handoff2026-11-05 (Europe/Berlin). Directory retains the run-start date.

**PASS for the tested S1 action/state/lifecycle contract at one output, scale1.** Both candidates completed the same functional sequence; all30 normalized domain/placement snapshots match. This is scoped functional evidence, not exact pixel/font equivalence, a completed usable shell, Gate R/P acceptance, production integration or renderer selection. Bounded visual differences and unverified behavior are recorded below. No S2–S6 work or performance measurements were executed.

## Authorization, source and implementation

Maintainer authorized S1 only from [the reviewed plan](../../SOURCE-REFERENCES.md#source-36d36ba96d923d46) at `7b83576ffd7e7bd028ae4270937ac06167ee373c`. The signed commit containing this report preserves the exact tested source overlay, implementation, raw evidence and review. [Provenance](../../SOURCE-REFERENCES.md#source-9351b4dc08beb6d0) binds modified sources, UI, lockfiles, fixture, driver, configuration and actual binary SHA-256:

`c878958ff15a92e9b6af81cdb3fa064c7d16900828d13d309df8ef463829465c`

[Implementation/build instructions](../../SOURCE-REFERENCES.md#source-86bbf6943147c0e4). One isolated executable selects exactly `OWLEN_RENDERER=slint` or `egui`; each ran separately in a fresh private session. Sharing the executable is convenient for functional testing and is not evidence about standalone dependency/memory footprints.

| Affected module | Implementation |
|---|---|
| `src/runtime/clock_widget.rs` | Adds explicit UTC `current_text_at` and focused regression; existing local-time default unchanged. The fixture uses the actual ClockWidgetController. |
| `src/runtime/coordinator.rs` | `pump` propagates `try_render_surface` errors. A failed visible GPU host cannot silently retire/recreate on repeated pumps. No recovery framework or fallback. |
| `experiment/functional-parity/src/model.rs` | Shared Core-side composition state holds real clock/calendar controllers, per-UI registration incarnations and a64-entry intent queue. It contains no Coordinator reference; renderer adapters contain no Coordinator reference. No Rc cycle or renderer-owned visibility/month model. |
| `src/main.rs` in that experiment | Owns the real Coordinator, explicit Core connection/dispatch/calloop, GenericAdapter, OutputManager and SurfaceManager. Applies `ShellSpec::default_shell`; calls actual controller actions through Coordinator; materializes hosts before attachment; performs ordered cleanup on normal exit and errors. No duplicate SCTK dispatcher. |
| `src/slint_ui.rs`, `ui/parity.slint` | Existing generated reference Bar/Calendar are imported with explicit public aliases and frozen font family. RenderBridge/SoftwareRenderer/wl_shm stay concrete. Callbacks enqueue intents instead of re-entering Core. Wrapper first-paint fault is explicitly synthetic. |
| `src/egui.rs`, `src/gpu.rs` | Two concrete per-host adapters and exclusive GPU presenters, not the old counter scene. Lazy binding to the first own-host Core-validated token; foreign/competing/stale token checks; release clears GPU epoch, not controller state. Retained Core display/layer handles and GPU-before-native teardown reuse the verified boundary. UI actions are queued after the final egui UI pass. |
| `fixtures/fp1.json`, `validation/` | Frozen dimensions/controller data/roles/font-byte bindings and scoped functional driver. Existing guardian, sealed executable pin, owned process groups, private socket checks and pointer fixture reused. No resource sampler or generic test framework. |

Root Cargo.toml/Cargo.lock, SCTK surface/output/generation/placement/persistence implementations, reference bar/calendar UI files, production Slint composition and original checkout are unchanged. Experimental dependencies remain in their own Cargo workspace/lock. Final graph pins Slint1.17.1, egui0.35.0, wgpu29.0.4, SCTK0.21.1 and wayland-client0.31.15, with system Wayland backend for raw handles.

## Shared behavior and ownership

The actual default ShellSpec supplies the32-logical-pixel output-width bar/exclusive32/keyboard None and320×320 calendar/exclusive0/OnDemand keyboard/default TopRight offset24,40. The calendar has six Monday-first rows/42 cells. The clock is frozen at UTC `2026-11-04T12:34:56Z`; month starts November2026 through CalendarWidgetController::with_date. Calendar data and HostState visibility/attachments/placement remain outside renderer presentation state.

Both adapters enqueue actual controller-produced WidgetAction/HostCommand values. The common loop completes the same Coordinator pump/input/drag/render phase, then drains valid UI actions outside borrows, applies them through Coordinator and synchronizes UI registration before the next attachment pump. This avoids egui callback reentrancy and aligns Slint input-time callbacks with egui paint-time callbacks. Logical intent survives native-only recreation; old UI-incarnation intents are rejected after unregister/re-register; overflow is terminal without Toggle coalescing. Focused tests exercise these policies and a real Coordinator mutable-borrow/action boundary.

Core retains native roles, connection, output tracking and generation decisions. Each surface generation uses exactly one presenter: Slint software/shm or egui/wgpu. The GPU selection cannot enter Core's shm path, even though that code remains compiled. Both bar and calendar may exist together, each with its own selected presenter. Renderer code does not attach/retire native surfaces; fixture recreation and re-registration use Coordinator commands.

## Runtime environment and acceptance results

Installed, unpatched Niri26.04 (8ed0da4), private headless Sway host, one winit output1276×691@60Hz, scale1. Each run has scrubbed environment, explicit private sockets and private HOME/XDG directories. GenericAdapter prevents active compositor IPC detection; no session readiness/services/D-Bus owners are started. Per-run environment/tool/config hashes and process identities are preserved. Private Niri diagnostic stayed disabled.

| Check | Slint | egui | Evidence / limit |
|---|---|---|---|
| Frozen bar/clock/calendar views | PASS scoped | PASS scoped | Eight complete compositor screenshots each;12:34:56, November/December title, dates, close and full bounds visible. No pixel-perfect claim. |
| Clock→Toggle, close→Hide | PASS | PASS |25 physical click sequences each, exactly one accepted controller action per click within the fixed guard, checked again after fixture cleanup for duplicates. First bar sequence is Enter→press→release without Motion, then leave; calendar initial close likewise enters then presses/releases. |
| Ten open/close cycles | PASS | PASS | `open-1..10`, `closed-1..10`; bar remains generation1; popup generations advance. No queued duplicate, RefCell reentry or leaked live host. |
| Clock-toggle dismissal | PASS | PASS | `clock-toggle-open/closed`, same actual HostState changes. |
| December survives hide/reopen | PASS | PASS | Controller changed to December through fixture, then hidden/reopened; month/title/rows remain December. This rejects a false reset-to-initial-month pass. |
| Native-only recreation | PASS | PASS | Coordinator Hide/Show without UI rematerialization; new native generation, same UI incarnation and December model. No manual token binding in runner. |
| UI unregister/re-register | PASS | PASS | Coordinator retires first, removes old UI, re-registers before next pump; newer UI/native incarnation, same December data; one deliberately queued old-UI dismiss rejected, popup remains visible. |
| Explicit logical reset | PASS | PASS | Real detach/attach widget API plus controller reset; only this operation returns November. |
| Domain/placement comparison | PASS | PASS | [Comparison](../../SOURCE-REFERENCES.md#source-9ef817af07db91c2): zero differences across30 matching state labels. Includes widgets, visibility, model data, saved placement, placement decision and active native geometry. |
| Native lifecycle/presentation | PASS scoped | PASS scoped |17 actual role/surface lifetimes each: stable bar +16 popup generations. Configure/ack precede first buffer; all role/surface destruction observed. egui GPU buffer destruction/release precedes native role/surface destruction; no shm pools in GPU run. |
| Ordered normal shutdown | PASS scoped | PASS scoped | Exit0, Coordinator hides hosts, adapters removed, native instances0, no layers and cleanup errors[]. This is the S1 runner's exit path, not full production signal/compositor-restart acceptance. |
| Calendar fault after successful bar | PASS injected policy | PASS injected policy | Both terminate after exactly one calendar attempt, exit1, no calendar frame, bar/native cleanup, no retry/fallback. Stages differ as detailed below. |
| Owned processes | PASS tested | PASS tested | Positive runs28 owned identities each; negative runs4 each; all64 recorded leaders/groups absent after cleanup. Trusted children only, not hostile daemonization/kernel D-state proof. |

[Offline assertions](../../SOURCE-REFERENCES.md#source-58695e1eb7526b5b), [read-only audit](../../SOURCE-REFERENCES.md#source-ca3f86ccc4782e13), [owned-group audit](../../SOURCE-REFERENCES.md#source-ea89c9be75ac40f8), [raw runs](../../SOURCE-REFERENCES.md#source-0328e897c05bae07) and [independent review](../../reviews/2026-11-04-s1-functional-parity.md). Raw client protocol establishes requests/events and buffer operations, while screenshots establish observed compositor output; neither proves physical display latency or authenticated server connection diagnostics.

### What the state comparison normalizes

It excludes renderer identity, wire object IDs, numeric UI/native epochs and backend buffer-mode flags; these necessarily differ and are checked separately per run. It retains registration presence, controller data, visibility, attached widgets, saved placement/decision and configured/requested/render dimensions, anchors, margins and scale. Numeric OutputIds are mapped to sole winit only after exact single-output preflight; no second output is silently normalized away. The actual private placement files are byte-identical across the two positive runs.

## Tests and independent verification

- Core suite:123 passed (including new deterministic clock regression), against the pinned experimental graph. Parent independently reran the full suite.
- Focused parity suite:11 passed, covering real controllers, bounded queue/overflow, incarnation rejection/state retention, actual Coordinator deferred-action borrow safety, egui token/input/header-close behavior and Slint wrapper full-size/callback projection. Parent reran it; independent reviewer reran exact built tests.
- Python functional-control/comparison tests:9 passed, including private path rejection, forbidden metric entry points, single-output checks, generation-scoped submission, exact exit/failure stage and close coordinates including exclusive32.
- Build, formatting/source diff checks and source/fixture hashes pass. Relevant old Slint/RenderBridge tests remain passing; no Clippy pass is claimed.

Logs are retained, including `owlen-s1-check-inherited.log`: the initial wrapper attempted inherited component properties not exported as public Rust methods. Explicit wrapper aliases fixed that compile error without changing reference UI. That failed compilation was never launched. Do not attribute final runtime results to the initial source.

## Recorded differences and unverified behavior

**Rendering:** requested semantic colors/geometry/glyph content and registered font bytes are bound in FP1. Representative composited samples mostly match exactly. At calendar bottom pixel(1000,380), Slint is(3,6,10), egui(2,5,9); [samples](../../SOURCE-REFERENCES.md#source-7d50ce5d369f5a99) preserve this rather than changing settings/tolerances after the run. Pinned egui-wgpu default dithering is enabled and its shader has a dithering branch, a plausible explanation not isolated per pixel. Antialiasing/text rasterization can also differ. No full-image, alpha-buffer or pixel equivalence is claimed.

**Fonts:** both variants register/request the same pinned DejaVu Sans regular/bold file pair, family and sizes. Slint day cells request numeric weight600; egui explicitly uses the registered bold face because this API has no numeric weight parameter. Clock/title/close use the documented bold mapping. Exact numeric requested-weight/per-glyph face equivalence is UNVERIFIED; complete glyph legibility and bounds were visually inspected. FP1 records the mapping; it is not proof that the font engines choose identical faces for every glyph. No system font settings were changed or font files vendored.

**Reference persistence side effect:** Core's existing header drag region includes the close button and commits placement even on a no-motion press/release. The common deferred-action phase lets both S1 variants complete that same Core phase before Hide, producing equivalent saved TopRight(24,40) placement. The pristine default Slint inline callback can hide/cancel before this phase. User-visible placement stays the same in these controls, but the experiment does not claim every persistence side effect matches pristine inline wiring. This is documented rather than fixed through S2 drag/capture redesign.

**Faults:** Slint uses a synthetic calendar first-paint wrapper fault, latched and checked immediately after pump before another attempt. It is not an induced Slint device/OOM failure. egui creates the calendar wgpu surface/device then injects `after-device`; partial resources drop before Core retires native objects. They test the same terminal/cleanup policy at different failure points, not equivalent failure mechanics. Real device loss, arbitrary backend faults, constructor failures outside existing tested rollback scope and full recovery remain unverified.

S1 has no new keyboard/scroll/focus semantics, fractional scaling, multi-output, output hotplug, deadlines/animation support or performance sampling. Renderer loops still use25ms control polling; no idle CPU/GPU/wakeup/energy efficiency follows. Hover/pressed mappings are source-checked and input transitions exercised; dedicated pressed-color screenshots and broad visual acceptance are not claimed. S1 is not the full functional-parity milestone, a daily-driver claim or production migration.

## Reproduction and preservation

The new package README contains locked build/test commands. Use private durable build storage, not the quota-constrained `/tmp` target. Exact commands, Rust toolchain, source hashes, build environment and runtime additions are in provenance. Both candidates used the same binary hash and same reviewed script, with explicit renderer selection.

Positive command shape, substituting `slint` then `egui` and separate fresh outputs:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 experiment/functional-parity/validation/validate.py \
  --renderer slint --mode functional \
  --binary <PRIVATE_PATH> \
  --sha256 c878958ff15a92e9b6af81cdb3fa064c7d16900828d13d309df8ef463829465c \
  --pointer /tmp/owlen-slint-boundary-validation/pointer-click/pointer \
  --pointer-sha256 7f18f66fa1b0666db9c900da7998302134e35ee356dce58513bc905c8019411f \
  --output /tmp/owlen-s1-slint-functional-01
```

The pointer fixture source/Makefile are preserved under `experiment/slint-reference-boundary/validation/pointer-click`; rebuild in a disposable directory and explicitly bind its resulting hash when reproducing. Use `--mode failure` and another fresh output for the targeted calendar fault; driver selects the documented renderer-specific fault environment. This reproduction procedure does not authorize benchmarks or active-desktop execution.

Existing evidence can be audited without a candidate launch:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 docs/bakeoffs/renderer/results/2026-11-04-s1-functional-parity/audit-evidence.py \
  docs/bakeoffs/renderer/results/2026-11-04-s1-functional-parity
```

Large raw client trace is losslessly gzip-compressed; [raw compression inventory](../../SOURCE-REFERENCES.md#source-e9317e8b50b64751) binds original uncompressed SHA-256/length. No trace was truncated to hide failures. Shader/build caches, runtime sockets and compiled binaries are excluded from Git. `/tmp` stayed above the1GiB reserve; [post-run storage](../../SOURCE-REFERENCES.md#source-765ae557735f9ec0) records it. The reviewer-created Python bytecode cache was removed; source/evidence remained unchanged.

The original checkout and its existing untracked handoff file were verified unchanged. No production merge, service/configuration change or push occurred. Historical NO-GOs remain valid, ADR-0006 stays Proposed, ADR-0007/evidence gates are unchanged, private Niri diagnostic disabled. Prior exploratory data remain permanently **EXPLORATORY — NON-DECISION-GRADE**. Renderer and architecture selection remain maintainer decisions. No winner has been selected.
