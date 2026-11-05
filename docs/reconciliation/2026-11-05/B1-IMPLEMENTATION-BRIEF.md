# Proposed B1 brief — concrete Slint ownership boundary

2026-11-05. **Planning draft; neither B1 implementation nor execution is authorized.** Depends on maintainer acceptance of the [B0 closure specifications](B0-GAP-DISPOSITION.md), [generation contract](B0-GENERATION-TOKEN-CONTRACT.md) and [retirement contract](B0-RETIREMENT-CONTRACT.md). This narrows G14; it does not replace the original milestone acceptance cases or complete Gate P.

## Proposed environment — requires maintainer confirmation

Recommend the smallest historically exercised one-output combination: **private headless Sway host → private nested Niri 26.04 (`8ed0da4`) → Slint 1.17.1 CPU software rendering → Core wl_shm**. Not the disabled private diagnostic Niri, not SwayFX substituted silently, and no egui/GPU API path. Current binary availability and a complete reproducible environment are **UNVERIFIED** in this task.

Historical [S1 report](../../bakeoffs/renderer/results/2026-11-04-s1-functional-parity/README.md) and its [provenance catalogue](../../bakeoffs/renderer/SOURCE-REFERENCES.md#source-9351b4dc08beb6d0) / [raw-run catalogue](../../bakeoffs/renderer/SOURCE-REFERENCES.md#source-0328e897c05bae07) identify:

| Item | Historical observation, not present-day verification |
|---|---|
| Compositor | Installed unpatched Niri 26.04 (`8ed0da4`); CLI and compositor agreed. Full source revision absent from inspected metadata. |
| Niri executable SHA-256 | `8ed88914b511a818fc55927d7992729f3652c4e10d777a1269b33f5d3428162d` |
| Private host | Headless Sway; executable SHA-256 `d7ec5122840f56f99d59cb426e830947105c499ed43902fa0a8370a08f6fc23c`. Human-readable version absent from inspected records. |
| Output | Sole `winit`, position (0,0), 1276×691 logical pixels, 60000 mHz, scale 1, recorded transform **Flipped180**. Do not silently report Normal. |
| Client dependencies | SCTK 0.21.1, wayland-client 0.31.15, calloop 0.14.4, Slint 1.17.1; canonical Cargo.lock agrees for these entries. |
| Source binding | S1 base `7b83576ffd7e7bd028ae4270937ac06167ee373c` plus enumerated source-hashed overlay, **not** an exact clean tested Git HEAD. It is not today's B1 source. |

Retained source at evidence revision `6304979719f1388e6ff98d549224107d0d45f284`: `docs/bakeoffs/renderer/results/2026-11-04-s1-functional-parity/provenance.json` and `runs/owlen-s1-slint-functional-01/environment.json`. Raw files are omitted from the curated canonical snapshot. Historical Niri/host config SHA-256 values are respectively `523ea91a005905ecb46bcab1b5c9e2e991c5d0b219e7d34f7d800859480eeb89` and `a25af0f2f1886a65071072881ac0aeb3733032ae109d999834c439f5eb8f0176`; these are provenance, not permission to run omitted commands.

Before any runtime authorization: bind the actual candidate source/lock/binary and available Niri/host binaries, document versions/build identities and any historical differences, confirm the chosen transform and effective geometry, and agree the exact command/timeout/cleanup plan. If the historical environment cannot be reconstructed, stop and propose an explicit replacement for approval. Never infer present identity from old hashes. Exactly **one active advertised client output**, verified by Niri effective IPC state and client registry, is mandatory; targeting one of two active outputs is insufficient. Host display geometry is not the candidate's output identity.

## Bounded implementation cut

Keep one Cargo package and one native/UI owner thread. Reuse WaylandPlatform, SurfaceManager, output registry, Coordinator, controllers, placement and concrete Slint/SHM code. No import of the experimental runtime wholesale.

| Existing modules | Required B1 responsibility / boundary |
|---|---|
| `src/main.rs`, `src/runtime/coordinator.rs` | Owner composition and canonical host/controller/attachment records; enqueue/apply typed actions without reentrant mutation. Remove direct Slint projection/materialization responsibilities from Core-facing paths. |
| `src/platform/render_bridge.rs`, existing compiled Slint components | Concrete per-host Slint integration: creation/registration identity, UI projection, neutral content sizing, input translation, dirty/toolkit facts and CPU paint. The module name is not a new generic API commitment. |
| `src/platform/wayland.rs`, `surface_manager.rs`, `output_manager.rs` | Preserve SCTK ownership; carry native origin and binding through queues; final admission checks; fix invalidate-before-drop; concrete configure/buffer submission; no toolkit-owned native surface. |
| `src/runtime/clock_widget.rs`, `calendar_widget.rs`, `src/domain/host.rs` | Preserve controller state and actions. Convert boundary strings/sizes/topology references to concrete neutral values where required; avoid unrelated domain rewrite. |
| Existing unit/integration tests | Add narrow identity/ordered-retirement/admission fixtures. Inject private state destinations/spies where needed; no real user-state writes. |

Minimal proposed calls exchange current registration/token, neutral view data and logical size, typed UI proposals, CPU paint output/dirty information, and explicit retire/rollback outcomes. Core alone decides native attach plan and SHM submission; the Slint boundary never chooses output, placement, logical visibility or persistence. Existing dirty/timer behavior must remain functional; complete scheduling/deadline redesign is B2. No multi-backend renderer trait, extra crate or service executor is needed.

## Composition, input and focus

Use [default_shell](../../../src/domain/default_shell.rs#L31): one bar host/clock widget and one initially hidden calendar host/widget. Bar is output-width ×32 logical pixels, top layer, exclusive zone32, keyboard None. Calendar is 320×320, TopRight offset (24,40), exclusive zone0, OnDemand keyboard; reconcile resolved placement/configure/buffer dimensions rather than invent another geometry authority. See [HostSpec](../../../src/domain/host.rs#L81), [bar height](../../../src/model.rs#L1) and [calendar root](../../../ui/calendar.slint#L37).

Clock/controller and calendar month remain Core-owned. Use deterministic controller dates/time only in tests; record fixture values. Runtime clock retains its declared one-second update behavior; B1 must not freeze it to manufacture equivalence or idle. No theme, asset or font replacement is part of the seam. Canonical calendar UI has only a dismiss callback; live month navigation in the historical S1 fixture is not current application functionality and is deferred/UNVERIFIED for B1. Retention uses the existing controller `set_date` in deterministic tests, not a newly invented visible control.

Minimum B1 input contract: record native enter coordinates before click-without-motion, press/release, motion and leave; one complete activation produces one Coordinator toggle; release after generation retirement cancels, never activates. Clear held/focus state only for its matching generation. Record duplicate native leave separately from semantic cancellation. Preserve working keyboard/scroll translation; no new claim that current UTF8-only keys or empty repeat implementation satisfy the broader product contract.

User-Open Focus remains unchanged: explicit clock activation may map OnDemand calendar and the compositor may focus it immediately. Focus grant is not guaranteed. If native keyboard enter is observed, test dismissal using an actually delivered supported key; otherwise record keyboard dismissal UNVERIFIED and use the visible close control. No invented keyboard capability or no-focus-on-map assertion. Unsolicited/background mapping is UNVERIFIED.

Expected B1 interaction regions are declared full rectangular host bounds, with interactive hit targets confined to visible clock/calendar controls. Check outside-surface clicks do not activate them. Transparent-edge custom input regions, non-text keys, repeat/capture, seat-loss and broad scaling quality remain G05/B3 gates; this limited rectangle declaration is not full P11 or invisible-edge acceptance.

## Required semantic scenarios and proposed tests

| ID | Sequence and objective acceptance | Evidence type |
|---|---|---|
| B1-01 | Construct canonical registries/attachments and bar/calendar UI; map bar; exactly one presenter for current token, no hidden calendar native submission | Registry/rollback tests + private runtime trace |
| B1-02 | Observe configure/ack, first submitted SHM buffer and frame callback separately, token-correlated; inspect actual bar geometry/content independently | Client/protocol trace + labeled compositor screenshot/manual inspection |
| B1-03 | Enter→click without motion→release once; one admitted clock action opens one calendar; close and reopen preserve controller month. Separately use a deterministic Core/controller date-change fixture to make retention observable; no live UI navigation is added | Raw input, action IDs, Coordinator before/after snapshots, visible controls |
| B1-04 | Native hide/recreate with retained logical state; fresh token/UI binding, no overlapping presenter; stale native/UI work has zero replacement effect | Deterministic negatives + bounded recreation trace |
| B1-05 | Live compositor Closed and duplicate terminal receipts; policy once before replacement, cleanup-only receipt afterward; old receipt cannot exit/ready/cancel replacement | Deterministic owner-loop interleaving tests; actual close if reliably observable |
| B1-06 | Failed creation after each acquired registration/native stage; matching-only reverse rollback; retry fresh identities | Fault-injected tests, not an unsupported claim of real device fault coverage |
| B1-07 | Model-only same-name delete/re-add, rehost A→B→A, model-only admitted controller update across native recreation, counter exhaustion | State/registration tests; no live dynamic composition or multi-output claim |
| B1-08 | Bounded ordinary close/teardown; presenter/native local release ordering; no owned survivors | Explicit local destruction records + independent harness cleanup receipt |

Run relevant existing Core/controller/platform/bridge/Coordinator suites and focused new tests **only under private HOME/XDG/TMPDIR and repository paths**. Existing `tests/coordinator.rs` can save to default state, so an unisolated cargo-test command is not acceptable. Record actual tests/run commands, failures and counts later; none are run by this brief. Do not call harness-forced cleanup graceful application shutdown. Full signal/lost-connection/phase-deadline shutdown acceptance belongs to B2.

## Readiness and runtime evidence

Use existing structured client logging, protocol capture and compositor diagnostics; no lifecycle diagnostic producer, full Wayland proxy or new acceptance framework. Each observation must include process/session, private socket, actual output, host registration/UI incarnation/token and event serial where available. Single-client isolation alone is not universal connection-scoped attribution; label client-side buffer/callback observations honestly. Any claim requiring absent server-side evidence remains UNVERIFIED.

Separate **configured**, **buffer-submitted**, **callback-observed** and **visual inspection** records. No final product `ready` trigger is selected by this brief. Correlate each protocol milestone to its generation; a missing callback or capture timeout is an incomplete observation, never visual PASS. Predeclare finite startup/action/cleanup bounds in the B1 execution proposal, justified from the selected harness and failure controls; do not add an arbitrary number here. Delayed/absent-callback tests must prove the bound fails honestly. G02's final readiness semantics and production deadline policy stay before B2.

Preflight must verify isolated parent/child runtime sockets and environment, no production socket/bus/service access, tracked process identities/groups, actual output identity/scale/transform and exclusive private state ownership. Preserve effective configs, binary/source hashes, raw input/state/lifecycle records, visual-inspection method and cleanup receipt. Screenshots are composition evidence, not alpha-buffer verification or display latency. No CPU/RSS/PSS/FPS collection.

## Exclusions, stop and recovery

No B2 global scheduler/shutdown implementation, B3 multi-output/scaling expansion, native fractional buffers, new Linux services, shared-state writer mechanism, configuration schema/GUI/IPC, live generic widget rehosting, GPU presenter, Slint hot-reload or production migration. B1 normal local cleanup/rollback is mandatory; it does not claim full B2 acceptance. Preserve S1–S6 historical limits and Niri multi-output BLOCKED.

Stop for ownership transfer, competing presenters, ambiguous origin, private isolation/cleanup failure, need to access real state/services, changed product focus/state semantics, or a requirement beyond this cut. Keep the existing composition as a comparison baseline; on a failed extraction preserve evidence and restore the known-good bounded source change only through an authorized ordinary revision, never reset unrelated work or deploy. A failed private test exits safely and reports its failed phase; no automatic restart/reconnect. All runtime actions and implementation still require separate maintainer authorization.
