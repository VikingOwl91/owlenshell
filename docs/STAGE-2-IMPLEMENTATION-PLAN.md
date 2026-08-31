# Stage 2 Implementation Plan

This plan turns the Stage 0/1 production domain and placement core into the
platform ownership model defined by `PRODUCTION-ARCHITECTURE-PLAN.md`. It is
grounded in commit `376cc0bfd72feebe1daa0c630da626b46a7e3358` and the current
proof-of-concept implementation.

Stage 2 is an ownership extraction, not a behavior or UI rewrite. The bar,
clock, calendar toggle, calendar dismissal, exact input geometry, semantic
placement persistence, output fallback and return-home, cross-output role
recreation, and current drop-on-release movement must remain observable. The
runtime remains one crate and one shell thread.

## Stage 2 Outcome

Stage 2 is complete when:

- raw Wayland and Smithay Client Toolkit objects exist only under `platform`;
- output inventory and topology ordering have one owner;
- native surfaces are keyed by `HostId`, not vector index or `SurfaceKind`;
- every native surface instance has a monotonic generation token;
- retired generations cannot mutate or signal a replacement surface;
- Slint window adapters are associated through an explicit `HostId` creation
  scope rather than the `new_adapters` stack;
- rendering still uses the custom Rust Slint platform, software renderer,
  `wl_shm`, configure-before-buffer behavior, and frame callback throttling;
- a thin coordinator resolves placement and issues typed platform commands;
- standalone and optional UWSM session behavior are represented behind a
  platform/session boundary;
- platform modules contain no calendar or bar branches; and
- all existing automated tests and explicit Wayland smoke scenarios pass.

## Non-Negotiable Behavior

The extraction must preserve these current behaviors unless a step explicitly
states that an internal diagnostic is being corrected:

1. The bar appears on startup, reserves its current top-edge area, and updates
   its clock every second.
2. Activating the bar calendar action creates the calendar; activating it
   again or dismissing the calendar hides and destroys its native surface.
3. Bar and calendar visible and input geometry remain exact. No invisible
   full-screen or oversized interaction surface may be introduced.
4. A layer surface receives its empty initial commit, waits for configure,
   acknowledges configure, and only then attaches a rendered buffer.
5. Rendering is blocked while a frame callback is pending.
6. Same-output drop updates margins; cross-output drop recreates the layer
   role against the selected `wl_output`.
7. Only the completed drop persists semantic output preference, anchor, and
   offset. Runtime fallback never overwrites saved intent.
8. A missing preferred output falls back without persistence and returns home
   when the preferred output is available again.
9. Output removal evicts the output before placement reconciliation.
10. Movement remains the proven fixed-surface, drop-on-release behavior.
    Continuous live movement belongs to Stage 4.

## Current Ownership Map

### Process Bootstrap: `src/main.rs`

`main.rs` currently owns too much platform setup:

- connects to Wayland and initializes the registry queue;
- binds compositor, layer shell, and shared memory globals;
- creates SCTK registry, seat, and output state;
- constructs `AppShared` and `AppState`;
- installs `OwlenPlatform`;
- inserts `WaylandSource` into calloop; and
- drives `pump` after each event-loop dispatch.

It also owns PoC composition and behavior: `Bar`, `Calendar`, calendar model
construction, clock updates, toggle/dismiss callbacks, and placement loading.
That application-specific logic may remain during Stage 2, but raw Wayland
bootstrap must leave this file.

### Runtime Monolith: `src/runtime.rs`

`runtime.rs` currently owns all of the following:

| Current type or function | Current responsibility | Stage 2 destination |
|---|---|---|
| `AppShared` | Wayland globals, native and logical surfaces, outputs, persistence, input, topology, compositor adapter, process flags | Split across platform managers and coordinator |
| `AppState` | SCTK dispatch state and raw pointer/keyboard objects | `platform/wayland.rs` |
| `SurfaceState` | Logical kind/visibility/placement plus native/render lifecycle | Logical compatibility record in coordinator plus native instance in SurfaceManager |
| `LiveSurface` | `wl_surface`, layer role, output, geometry, shm pool/buffer | `platform/surface_manager.rs` with render state from `platform/render_bridge.rs` |
| `DragState` | Indexed input session and raw runtime output | Temporary compatibility input state keyed by `HostId` |
| `OwlenPlatform` | Slint platform and positional adapter side channel | `platform/render_bridge.rs` |
| `drain_and_attach` | Pops adapter, assigns vector index, creates/reuses `SurfaceState` | Explicit HostId creation handshake plus coordinator registration |
| `hide_kind` | Kind lookup, native teardown, input cleanup | Coordinator host command plus SurfaceManager retirement |
| `pump` | Topology refresh, reconcile, attach, size sync, draw, flush | Coordinator ordering plus platform methods |
| `refresh_outputs` | SCTK fact extraction, native map, adapter enrichment, canonical snapshots | `platform/output_manager.rs` |
| `reconcile` | Placement resolution and native invalidation | `runtime/coordinator.rs` |
| `try_attach` | Placement lookup and hardcoded layer-surface construction | Coordinator-created generic surface spec applied by SurfaceManager |
| `sync_size`, `draw_surface` | Slint size/scale and shm rendering | `platform/render_bridge.rs` |
| SCTK handler impls | Protocol callbacks and direct runtime mutation | `platform/wayland.rs`, routed through managers |
| pointer/keyboard handlers | Native translation, focus, calendar drag policy | Native translation in platform; temporary policy in coordinator compatibility layer |
| `finish_drag` | Pure move commit, persistence, same/cross-output mutation | Coordinator, issuing typed SurfaceManager command |

The current `Vec<SurfaceState>` and `OwlenWindowAdapter.surface:
Cell<Option<usize>>` make vector position a cross-subsystem identity. Native
callbacks locate a surface by scanning raw object equality. This is the main
identity and stale-callback risk Stage 2 removes.

### Legacy Model: `src/model.rs`

`model.rs` re-exports domain types but also contains PoC constants,
`SurfaceKind`, and the conversion from domain `Anchor` to SCTK `LayerAnchor`.
The raw protocol conversion must move to `platform/surface_manager.rs`.
`SurfaceKind`, `CALENDAR_KEY`, and the current size/input constants may remain
temporarily in a compatibility module used by bootstrap and coordinator. They
must not be imported by platform modules.

### Window Adapter: `src/window.rs`

`OwlenWindowAdapter` owns Slint's `MinimalSoftwareWindow`, preferred size,
visibility, and a mutable optional surface vector index. Stage 2 moves this
type under `platform/render_bridge.rs`, replaces the index with immutable
`HostId`, and registers adapters by host identity.

### Output and Compositor Code

`refresh_outputs` in `runtime.rs` currently composes both raw `wl_output`
objects and domain `OutputSnapshot` values. `src/compositor.rs` mutates those
snapshots and may synchronously invoke `hyprctl`. Stage 2 moves the compositor
adapter behind `platform/compositor.rs` and gives OutputManager sole authority
to compose canonical snapshots. Replacing synchronous adapter polling is not
required in this stage; isolating it so it cannot own inventory is required.

### Pure Core and Persistence

`src/domain/**`, `src/runtime/placement_engine.rs`, and
`src/persistence/{schema,repository}.rs` are the Stage 0/1 baseline. They stay
free of Wayland and Slint. Placement remains a pure decision. Persistence
remains owned above the platform boundary and stores only saved user intent.

## Target Stage 2 Ownership Map

The target remains one crate:

```text
src/
  main.rs
  lib.rs
  app.rs                              composition root and calloop loop

  domain/                             unchanged pure production vocabulary

  platform.rs
  platform/
    wayland.rs                        connection, globals, queue and SCTK dispatch
    output_manager.rs                 native output registry and canonical topology
    surface_manager.rs                HostId-keyed native surface lifecycle
    render_bridge.rs                  Slint adapters and wl_shm rendering
    compositor.rs                     compositor enrichment adapter boundary
    session.rs                        session mode, capabilities and interface
    session/
      standalone.rs                   safe default implementation
      uwsm.rs                         optional backend boundary, no frozen mechanics

  runtime.rs
  runtime/
    placement_engine.rs               existing pure engine
    coordinator.rs                    logical reconciliation and persistence triggers
    poc_compat.rs                     temporary Bar/Calendar policies and drag behavior

  persistence/                        unchanged Stage 1 boundary
```

`app.rs` is optional as an intermediate extraction file if introducing it in
the same step would make `main.rs` smaller without moving application logic
into platform. The required boundary is that `main.rs` no longer imports
Wayland or SCTK. Do not create empty placeholder modules merely to match this
tree.

### Platform Root

The concrete platform root owns `WaylandState`, OutputManager,
SurfaceManager, RenderBridge, the compositor adapter, and a typed event queue.
It exposes direct, narrow methods rather than a broad object-safe abstraction
over all Wayland behavior.

Conceptual boundary:

```rust
pub struct WaylandPlatform { /* raw ownership is private */ }

pub enum PlatformEvent {
    TopologyChanged(TopologySnapshot),
    Surface(SurfaceEvent),
    Input(InputEvent),
    ExitRequested,
}

impl WaylandPlatform {
    pub fn connect() -> Result<(Self, WaylandSourceParts), PlatformError>;
    pub fn drain_events(&mut self) -> impl Iterator<Item = PlatformEvent>;
    pub fn outputs(&self) -> &TopologySnapshot;
    pub fn apply_surface(&mut self, command: SurfaceCommand) -> Result<(), PlatformError>;
    pub fn render_dirty(&mut self) -> Result<(), PlatformError>;
    pub fn flush(&self) -> Result<(), PlatformError>;
}
```

Exact calloop wrapper types should follow the APIs available in the pinned
dependencies. The design requirement is ownership, not the names in this
sketch. `WaylandSource` may require the constructor to return connection and
event-queue parts for insertion while `WaylandState` remains the callback
state.

### OutputManager

OutputManager is the only owner of output inventory. It maintains:

```text
OutputManager
  next_output_id: monotonically increasing counter
  topology_generation: monotonically increasing counter
  native_by_id: OutputId -> NativeOutputEntry
  id_by_native_identity: internal Wayland identity -> OutputId
  snapshot: TopologySnapshot
  pending protocol updates
```

`NativeOutputEntry` is private to platform and may contain `wl_output`, SCTK
protocol facts, and adapter enrichment facts. `TopologySnapshot` contains only
domain-safe `OutputSnapshot` values and a `TopologyGeneration`.

#### Runtime Output IDs

`OutputId` remains runtime-only and non-serializable. OutputManager allocates
it monotonically instead of reusing SCTK's `OutputInfo.id`. A disconnected
output's ID is never reassigned in the process. Stable output intent continues
to use `OutputPreference` evidence, not `OutputId`.

#### Topology Commit Rules

1. Protocol callbacks update a pending native record.
2. A complete output update composes protocol facts and optional compositor
   enrichment into one canonical `OutputSnapshot`.
3. If canonical inventory changes, OutputManager increments
   `TopologyGeneration` once for that committed change and publishes one typed
   topology event.
4. Removal is exceptional: evict the native routing entry and canonical
   snapshot synchronously before any coordinator reconciliation. Then advance
   topology generation and publish removal.
5. Removed OutputIds cannot appear in the snapshot passed to PlacementEngine
   or be accepted by SurfaceManager create/recreate commands.
6. Adapter enrichment never inserts, removes, or mutates inventory directly.
   It returns facts that OutputManager merges with explicit precedence.

#### Geometry Readiness

An advertised output without logical geometry is not placement-capable. It may
remain visible in diagnostic inventory, but the coordinator must pass only
placement-capable snapshots to `PlacementEngine::resolve`. If no output has
logical size, placement is `Deferred`; no layer surface is created. This fixes
the current contradictory `Placed { is_reachable: false }` lifecycle without
changing persisted intent.

`logical_position` is not required for ordinary anchored placement, but it is
required for cross-output drop hit testing. If positions are unavailable or
ambiguous, the existing conservative behavior keeps the source output.

#### Diagnostics

The topology event and placement trace must include topology generation,
selected runtime OutputId when placed, `ResolveReason`, fallback status, and
geometry readiness. The reason reported must be the reason for the target
actually applied. A stale decision against an older topology generation must
be resolved again rather than relabeled.

### SurfaceManager

SurfaceManager owns native instances only:

```text
SurfaceManager
  hosts: HostId -> SurfaceSlot
  route_by_wl_surface: native object identity -> SurfaceToken
  route_by_layer_surface: native object identity -> SurfaceToken

SurfaceSlot
  next_generation
  current: Option<AtomicSurface>

SurfaceToken
  host_id
  generation

AtomicSurface
  token
  output_id
  lifecycle
  wl_surface
  layer_surface
  requested geometry and input policy
  configured geometry
  render_surface
```

It does not own `HostState`, `WidgetState`, `SavedPlacement`, calendar state,
or persistence.

#### Generation Rules

1. Every HostId has a generation counter that survives periods with no native
   instance.
2. Publishing a newly created native instance consumes the next generation.
   Initial create, output recreation, hide/show recreation, and replacement
   after close all receive distinct increasing generations.
3. A generation never resets because a surface is hidden or destroyed.
4. Callback routing captures `{ host_id, generation }` at native registration.
5. Before handling configure, frame, scale, close, or input against a surface,
   SurfaceManager verifies that both token and raw object identity match the
   current instance.
6. Retirement removes all native routing entries before the layer role and
   `wl_surface` are dropped.
7. A callback with no route or a retired generation is ignored and traced as
   stale. It cannot change configure state, clear `frame_pending`, alter focus,
   or request logical host teardown.

Use a small pure `SurfaceRegistry` state machine for generation allocation and
token validation so stale-event behavior can be unit-tested without Wayland.
Raw handles stay in the concrete manager around that state machine.

#### Lifecycle Commands

The coordinator addresses all commands by `HostId`:

```rust
pub enum SurfaceCommand {
    EnsureVisible { host_id: HostId, spec: SurfaceSpec, placement: PlacedSurface },
    Reconfigure { host_id: HostId, spec: SurfaceSpec, placement: PlacedSurface },
    RecreateOnOutput { host_id: HostId, spec: SurfaceSpec, placement: PlacedSurface },
    Hide { host_id: HostId },
    Destroy { host_id: HostId },
    RequestFrame { host_id: HostId },
}
```

`SurfaceSpec` is a temporary generic native policy value: namespace, layer,
anchor, size policy, margins/input policy, keyboard interactivity, and
exclusive zone. It must not contain `SurfaceKind`. Stage 2 may construct specs
from temporary Bar/Calendar compatibility code. Stage 3 replaces that factory
with generic host runtime policy.

#### Cross-Output Recreation

For a target OutputId change:

1. Coordinator preserves logical host state, Slint component, adapter, and
   saved placement.
2. SurfaceManager validates the target against current OutputManager inventory.
3. SurfaceManager retires generation N and removes its callback routes.
4. It creates generation N+1 against the target `wl_output`.
5. It performs the required empty commit.
6. Configure for N+1 initializes configured geometry and rendering.
7. Events from N are rejected.

If replacement creation fails, the manager reports a typed failure. The
logical host remains present and the coordinator may defer/retry; it must not
persist fallback or lose widget state.

### HostId-Keyed Slint Adapter Creation

Slint's `Platform::create_window_adapter` callback does not carry application
identity. The current code compensates with a global vector and `pop`, which
assumes creation order. Stage 2 uses a synchronous scoped identity handshake.

RenderBridge owns:

```text
pending_creation: None | PendingWindowCreation { host_id, nonce }
adapters: HostId -> Rc<OwlenWindowAdapter>
next_nonce
```

The composition path is conceptually:

```rust
let bar = render_bridge.with_host_creation(bar_id.clone(), || Bar::new())?;
```

`with_host_creation`:

1. Rejects a nested pending creation.
2. Rejects creation for a HostId with an active adapter.
3. Installs a unique pending token.
4. Runs the Slint component constructor synchronously.
5. `create_window_adapter` consumes exactly that pending token and constructs
   `OwlenWindowAdapter { host_id, ... }`.
6. Registers the adapter by HostId.
7. Verifies that the constructor consumed the token exactly once.
8. Clears the pending token on every error path. Prefer an RAII guard so panic
   unwinding cannot poison later creations.

`create_window_adapter` without a pending token, a second callback for one
token, duplicate HostId registration, and nested creation are errors rather
than positional guesses.

An early extraction test must prove that the pinned Slint version invokes
`create_window_adapter` synchronously during `Bar::new()` and
`Calendar::new()`. If it is lazy, replace the single pending slot with an
explicit request-token map tied to the component creation call. Do not restore
a FIFO/LIFO queue.

The adapter's HostId is immutable. Native surface recreation reuses the same
logical adapter and Slint component; it does not invoke component construction
or adapter creation again.

### RenderBridge

RenderBridge owns:

- `OwlenPlatform` and `OwlenWindowAdapter`;
- HostId adapter registry and creation scope;
- Slint event dispatch and preferred-size observation;
- scale dispatch and software-window physical size;
- software renderer invocation;
- `wl_shm` slot pool and buffer state associated with a native generation;
- dirty state and frame-pending transitions.

The native `AtomicSurface` may contain a `RenderSurface` subrecord so raw shm
objects share the same lifetime as the native generation. Keep methods narrow
to avoid cyclic manager ownership: SurfaceManager owns the instance and calls
RenderBridge functions that operate on its render subrecord.

Preserve the current render protocol:

1. Do not render before configure.
2. Do not render while `frame_pending` is true.
3. Dispatch scale changes to Slint before calculating physical size.
4. Keep exact configured/requested geometry and the current software renderer.
5. Render to the current fresh pixel vector and copy through `SlotPool`.
6. Attach, damage, request a frame callback, commit, and set frame pending.
7. Clear frame pending only for the matching current generation's frame
   callback.

Reusable buffers, fractional scaling, and performance redesign remain Stage 7.

### Coordinator After Extraction

`runtime/coordinator.rs` owns logical reconciliation, not protocols. Its state
is keyed by `HostId` and contains temporary compatibility metadata,
visibility, optional `SavedPlacement`, current `PlacementDecision`, and the
topology generation against which that decision was made.

Its ordered cycle is:

1. Drain typed platform events.
2. Apply the newest committed topology snapshot.
3. Update logical visibility, input, and compatibility state.
4. Resolve affected hosts with `PlacementEngine::resolve(saved.as_ref(), ...)`.
5. Compare decision generation and current native target.
6. Issue typed create, reconfigure, recreate, hide, or destroy commands.
7. Ask the platform to render dirty current generations.
8. Persist only explicit completed move commits.
9. Flush platform output.

The coordinator must not:

- import Wayland/SCTK types;
- convert anchors into layer-shell anchors;
- retain `wl_output`, `wl_surface`, layer roles, buffers, or callbacks;
- index hosts by vector position;
- calculate calendar data;
- branch on Calendar or Bar when applying platform operations; or
- persist a placement decision merely because fallback occurred.

Temporary `runtime/poc_compat.rs` may map `SurfaceKind` to HostId and
`SurfaceSpec`, preserve the current bar first-frame milestone and
bar-layer-close process exit behavior, and contain the calendar-only drop
interaction until Stages 3 and 4. Its temporary status must be explicit.
Platform code receives only HostId and generic policy values.

### Stage 0/1 Follow-Ups Included in Stage 2

Three prior follow-ups fit this extraction naturally:

1. Remove the compatibility resolver in `src/placement.rs` that manufactures
   `SavedPlacement` from an `OutputPreference`. Coordinator calls
   `PlacementEngine::resolve(Option<&SavedPlacement>, ...)` directly so
   diagnostics distinguish no saved intent from fallback away from saved
   intent.
2. Treat outputs without logical size as not placement-capable. Do not create
   a native surface for a `Placed` result that cannot be reached.
3. Tie placement diagnostics to topology generation and the decision actually
   applied. Re-resolve stale decisions rather than logging a reason for a
   discarded target.

Partial output fingerprint matching policy remains deferred. Stage 2 must not
silently strengthen partial make/model/size evidence.

### Session and UWSM Boundary

Session integration is a platform concern, first-class and optional. Domain,
widgets, placement, and persistence must not know whether UWSM is present.

Stage 2 establishes vocabulary and safe selection:

```rust
pub enum SessionMode {
    Standalone,
    Managed(SessionManagerKind),
}

pub struct SessionCapabilities {
    pub lifecycle_notifications: bool,
    pub managed_application_launch: bool,
}

pub trait SessionIntegration {
    fn mode(&self) -> SessionMode;
    fn capabilities(&self) -> SessionCapabilities;
    fn notify_ready(&mut self) -> Result<(), SessionError>;
    fn begin_shutdown(&mut self) -> Result<(), SessionError>;
    fn launch(&mut self, request: ApplicationLaunch) -> Result<ChildHandle, SessionError>;
}
```

Names and concrete process handle types may follow implementation constraints.
The contract is capability-based behavior and standalone parity.

`standalone.rs` is the safe default and can directly launch processes when a
launcher exists. `uwsm.rs` is an optional backend boundary selected only from
verified detection evidence. Stage 2 must not freeze undocumented environment
variables, systemd unit names, readiness commands, or `uwsm app` invocation
details as stable APIs. Detection and process execution must be injectable in
tests.

No launcher call site needs to be invented in Stage 2. The boundary can be
constructed and tested while current shell behavior remains unchanged. Full
managed-session lifecycle and application-launch validation is a Stage 7 gate.
Failure to detect or use UWSM must fall back to standalone operation unless an
explicit managed-only operation was requested; Owlenshell itself must still
start and function without UWSM.

## Incremental Migration Sequence

Every step should be independently reviewable and leave the tree compiling.
Do not start two steps that write the same ownership boundary concurrently.

### Step 1: Add Platform Vocabulary and Pure Lifecycle Models

**Purpose:** Establish typed commands, events, topology generation, surface
tokens, and pure registries before moving raw objects.

**Files:**

- add `src/platform.rs`;
- add `src/platform/output_manager.rs` with pure topology state first;
- add `src/platform/surface_manager.rs` with pure generation registry first;
- update `src/lib.rs`;
- add focused unit tests, preferably module tests or `tests/platform_state.rs`.

**Move/change:** No raw ownership yet. Define `TopologyGeneration`,
`SurfaceGeneration`, `SurfaceToken`, typed events/commands, and state-machine
rules.

**Unchanged behavior:** Entire PoC still runs through `runtime.rs`.

**Tests/gate:** Generation increments and never resets; stale and mismatched
tokens reject; output removal precedes published snapshot; runtime OutputIds
are not reused; no Wayland connection required.

**Failure modes:** Public types accidentally expose raw Wayland handles;
generation stored only in current instance and reset after hide; command enums
encode `SurfaceKind`.

### Step 2: Move Compositor Adapter Behind Platform

**Purpose:** Put compositor-specific enrichment under the platform namespace
before OutputManager assumes inventory ownership.

**Files:**

- move `src/compositor.rs` to `src/platform/compositor.rs`;
- update imports in `src/runtime.rs` and `src/lib.rs`.

**Move/change:** Preserve backend detection and current enrichment behavior.
Change the interface only as needed so it returns enrichment facts instead of
owning canonical output snapshots.

**Unchanged behavior:** Hyprland enrichment and generic fallback produce the
same observed output facts.

**Tests/gate:** Existing tests pass; adapter cannot add/remove canonical
outputs; platform is the only public path to compositor adapter types.

**Failure modes:** Expanding scope into asynchronous process management;
changing matching precedence; adapter retaining stale inventory.

### Step 3: Extract OutputManager and Topology Ordering

**Purpose:** Establish one owner for raw outputs, runtime IDs, snapshots, and
removal ordering.

**Files:**

- complete `src/platform/output_manager.rs`;
- modify `src/runtime.rs` to delegate current refresh/removal paths;
- extend platform state tests and placement tests.

**Move/change:** Move `wl_outputs`, output snapshot composition,
`refresh_outputs`, adapter merge, and topology dirty/commit behavior into
OutputManager. Allocate monotonic OutputIds. Expose placement-capable output
views and topology generation.

**Unchanged behavior:** Matching, fallback, return-home, and cross-output drop
selection remain semantically the same for fully described outputs.

**Tests/gate:** Add/update/remove ordering; removal unavailable before
reconcile; incomplete geometry deferred; adapter enrichment merge; deterministic
snapshot; no removed ID accepted; existing 28 placement tests remain green.

**Failure modes:** Mapping runtime ID from connector name; reusing IDs after
disconnect; publishing an updated snapshot before native routing exists;
keeping geometry-incomplete outputs in placement candidates.

### Step 4: Extract RenderBridge and Prove HostId Handshake

**Purpose:** Remove positional Slint adapter association without changing
native surface ownership yet.

**Files:**

- add `src/platform/render_bridge.rs`;
- move and then remove `src/window.rs`;
- modify Slint platform setup in `src/main.rs`/`src/runtime.rs` temporarily;
- add render bridge identity tests.

**Move/change:** Move `OwlenPlatform`, `OwlenWindowAdapter`, adapter state,
Slint size/scale/event dispatch, and scoped HostId creation registration.
Replace `new_adapters`, `drain_and_attach`, and adapter surface index with
HostId lookup.

**Unchanged behavior:** Bar and calendar component construction, preferred
sizes, visibility, input dispatch, and visuals.

**Tests/gate:** Pinned Slint constructors consume the expected token
synchronously; sequential hosts bind correctly; duplicate/nested/missing
tokens fail; adapter survives native recreation; no queue/pop association
remains.

**Failure modes:** Assuming synchrony without a test; clearing pending state
only on success; reconstructing Slint components during output recreation.

### Step 5: Extract SurfaceManager and Generation-Safe Routing

**Purpose:** Move native role lifecycle from indexed `SurfaceState` into a
HostId-keyed manager.

**Files:**

- complete `src/platform/surface_manager.rs`;
- modify `src/runtime.rs` callbacks and attachment paths;
- move Anchor-to-LayerAnchor conversion out of `src/model.rs`;
- add lifecycle tests.

**Move/change:** Move `LiveSurface`, layer role creation, native output lookup,
native routing maps, configure/frame/scale/close handling, role retirement,
and render subrecords. Replace raw object scans with SurfaceToken routing.

**Unchanged behavior:** Layer namespaces, anchors, sizes, margins, keyboard
interactivity, exclusive zones, configure sequence, rendering, hide/show, and
cross-output recreation.

**Tests/gate:** Initial/recreated generations differ; callback from retired
generation cannot mutate replacement; routes removed before retirement;
same-output reconfigure does not unnecessarily recreate; cross-output does;
unknown target OutputId is rejected.

**Failure modes:** Dropping old role before deleting routes; resetting counter
on hide; tying adapter lifetime to native generation; platform branching on
Bar/Calendar to construct policy.

### Step 6: Move SCTK Dispatch and Bootstrap into Wayland Platform

**Purpose:** Complete raw Wayland ownership under `platform`.

**Files:**

- add `src/platform/wayland.rs`;
- modify `src/main.rs` and `src/runtime.rs`;
- update `src/lib.rs`;
- remove obsolete raw imports and state.

**Move/change:** Move connection, registry initialization, global binding,
`AppState`, raw seat devices, SCTK delegate implementations, input
translation, queue handle, and flush behavior. Expose typed events and a
calloop insertion boundary.

**Unchanged behavior:** One pointer/keyboard limitation, current modifiers,
focus behavior, calendar drop initiation, timers, and event-loop cadence.

**Tests/gate:** `main.rs`, domain, persistence, and coordinator contain no
Wayland/SCTK imports; all protocol callbacks route through managers; current
Wayland smoke starts, configures, renders, accepts input, and exits cleanly.

**Failure modes:** Broad platform trait obscures callback ownership; calloop
lifetime forces raw queue objects back into `main`; native input event loses
HostId/generation before delivery.

### Step 7: Introduce the Thin Coordinator and Remove Compatibility Resolver

**Purpose:** Separate logical decisions and persistence from protocol
application.

**Files:**

- add `src/runtime/coordinator.rs`;
- add `src/runtime/poc_compat.rs` if needed;
- modify `src/runtime.rs`, `src/placement.rs`, `src/main.rs`, and `src/lib.rs`;
- add coordinator tests.

**Move/change:** Move `pump`, reconciliation, visibility, placement loading,
completed-drop persistence, and same/cross-output command selection into
coordinator. Call PlacementEngine directly with optional saved intent. Filter
placement-capable outputs and retain topology generation with decisions.
Replace indexed drag state with HostId while preserving fixed-surface motion.

**Unchanged behavior:** All current user-visible PoC behavior and calendar
logic location except what is strictly needed to address hosts by HostId.

**Tests/gate:** No-saved-intent reason differs from saved fallback; stale
topology decision re-resolves; fallback does not persist; explicit drop
persists once; return-home issues recreate; hidden logical state survives
native destruction; coordinator contains no raw Wayland types.

**Failure modes:** Coordinator becomes a renamed monolith; persistence enters
SurfaceManager; temporary compatibility policy leaks into platform; topology
events and direct snapshot reads create duplicate truth.

### Step 8: Establish Session/UWSM Boundary

**Purpose:** Make optional managed-session support architecturally real without
prematurely committing to unverified mechanics.

**Files:**

- add `src/platform/session.rs`;
- add `src/platform/session/standalone.rs`;
- add `src/platform/session/uwsm.rs` only when it contains a tested boundary,
  not speculative commands;
- wire selection in platform/app composition root;
- add pure selection/capability tests.

**Move/change:** Define mode, capabilities, lifecycle, and application-launch
interface. Implement safe standalone selection and test-injected detection.

**Unchanged behavior:** Owlenshell startup and current shell interactions do
not depend on UWSM. No launcher feature is introduced.

**Tests/gate:** Absence/failed detection selects standalone; managed selection
does not leak outside platform; domain/widget modules do not import session
types; fake managed backend can prove delegation without invoking UWSM.

**Failure modes:** Treating UWSM as mandatory; encoding guessed environment
variables as API; adding process-spawning behavior with no current caller;
placing session capability in HostSpec or widget state.

### Step 9: Delete Old Ownership and Run Full Regression

**Purpose:** Ensure extraction is complete rather than leaving two paths.

**Files:**

- reduce or remove legacy `src/runtime.rs`, `src/window.rs`, and
  `src/compositor.rs` as their contents move;
- update module exports and documentation references;
- do not alter Slint files unless a build-only import path change is necessary.

**Move/change:** Delete `AppShared`, indexed `SurfaceState`, `LiveSurface`,
`new_adapters`, positional adapter fields, raw output maps outside platform,
and the manufactured-intent compatibility resolver.

**Unchanged behavior:** Entire PoC interaction contract listed above.

**Tests/gate:** Raw-import audit, full Rust gates, and manual compositor smoke
matrix all pass. A final diff audit finds no Stage 3+ implementation.

**Failure modes:** Dead compatibility path remains callable; duplicate output
or surface stores survive; docs claim live movement or UWSM validation that
was not exercised.

## Testing Strategy

### Pure Automated Tests

Add tests that require no Wayland compositor:

| Area | Required cases |
|---|---|
| OutputManager state | monotonic IDs, no reuse, add/update/remove, topology generation, removal ordering, deterministic snapshots, enrichment merge |
| Geometry readiness | advertised without size excluded from candidates, all incomplete means deferred, logical position absence preserves conservative drop behavior |
| Surface registry | first generation, recreate increment, hide/show increment, stale token rejection, object/token mismatch rejection, route removal |
| Adapter creation | exact HostId binding, sequential creation, duplicate HostId, missing token, nested token, cleanup after constructor error |
| Coordinator | direct optional saved intent, fallback without persistence, return-home, stale topology re-resolution, same-output reconfigure, cross-output recreate, hide preserves logical state |
| Move compatibility | drop persists once, target OutputId retained through command, no intermediate persistence, removed target rejected |
| Session selection | standalone default, failed detection fallback, managed capabilities, fake launch/lifecycle delegation, no domain coupling |

Keep the existing 42 Stage 0/1 tests green. Extend tests rather than weakening
assertions to accommodate the extraction.

### Compile-Time and Static Boundary Checks

At each relevant step run grep-based audits in addition to compiler checks:

- `wayland_client` and `smithay_client_toolkit` imports occur only under
  `src/platform/**`;
- `SurfaceKind`, `CALENDAR_KEY`, `Bar`, and `Calendar` are absent from
  `src/platform/**`;
- `serde` is not added to runtime-only `OutputId`, generations, or raw platform
  records;
- `wl_output`, `wl_surface`, `LayerSurface`, `SlotPool`, and `Buffer` do not
  appear in coordinator/domain/persistence;
- the adapter registry contains no vector-position identity or `pop`-based
  association.

### Wayland Smoke Scenarios

Pure tests cannot validate protocol ordering. Exercise these scenarios on a
supported compositor, recording compositor and environment:

1. Cold start with one output: bar configures, first buffer appears, clock
   updates, and no buffer is attached before configure.
2. Open/dismiss/reopen calendar: each native instance configures and renders;
   logical interaction remains intact.
3. Same-output calendar drop: visible position updates and restart restores
   semantic placement.
4. Cross-output calendar drop: old generation retires, replacement configures,
   and calendar state does not reset.
5. Disconnect preferred output while calendar is visible: removal is applied
   before reconciliation; fallback appears if placement-capable output exists;
   persisted home remains unchanged.
6. Reconnect preferred output: host returns home through controlled
   recreation.
7. Start with no outputs or only geometry-incomplete outputs, then add a usable
   output: logical hosts defer safely and later become visible.
8. Trigger redraw pressure: only one frame callback remains pending per current
   generation and idle rendering stays throttled.
9. Run outside UWSM: startup and every existing shell interaction remain
   functional.

Where possible, add a nested-compositor integration harness later. Stage 2
must not represent unexercised protocol behavior as automated coverage.

## Acceptance Gates

### Architecture Gates

- Raw Wayland/SCTK types and imports exist only in `src/platform/**`.
- OutputManager is the sole canonical output inventory owner.
- SurfaceManager's public operations are keyed by HostId.
- Runtime OutputId and generation tokens are not serialized.
- Native callback routing validates HostId, generation, and object identity.
- Adapter association is explicit and HostId-keyed, with no queue-position
  fallback.
- Coordinator owns saved placement and persistence triggers; platform owns
  neither.
- Platform modules have no `SurfaceKind`, Bar, Calendar, date, or widget
  behavior branches.
- Session/UWSM types remain under platform and standalone remains default-safe.
- No new crate is introduced.

### Behavioral Gates

- Existing bar/calendar/clock interaction and visuals are unchanged.
- Exact visible geometry equals input geometry.
- Configure-before-buffer and frame throttling are preserved.
- Same-output and cross-output completed drops behave as before.
- Fallback never persists; preferred-output return-home still occurs.
- Removed outputs cannot win placement or receive effective stale callbacks.
- Logical host/Slint state survives native recreation.
- Drop-only movement remains; no false claim of production live movement is
  introduced.

### Required Commands

Run from repository root after each meaningful extraction step, and all of
them before Stage 2 is declared complete:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
git diff --check
```

The final acceptance record must include exact test counts, manual smoke scope,
compositor used, and any skipped scenario.

## Risks and Mitigations

| Risk | Consequence | Mitigation |
|---|---|---|
| Slint adapter creation is not synchronous | Scoped HostId token cannot bind reliably | Prove behavior first against pinned Slint; use explicit request-token map if lazy; never use queue order |
| Borrowing between SCTK callbacks and managers becomes cyclic | RefCell panics or forced god object | Keep raw dispatch in one concrete WaylandState; use short manager borrows and typed queued events |
| Old callback mutates replacement | Frame/configure/focus corruption after recreation | Route by object to SurfaceToken, remove routes before drop, validate current generation on every callback |
| Runtime OutputId accidentally becomes stable identity | Wrong restoration or ID reuse after hotplug | Allocate monotonically, keep non-Serde, persist only OutputPreference evidence |
| Incomplete output geometry creates unreachable native surface | Surface is selected but cannot be safely placed | Separate diagnostic inventory from placement-capable view and defer when none are usable |
| Removal ordering regresses | Removed output wins reconciliation | Synchronous eviction and topology commit before publishing event or invoking coordinator |
| Coordinator becomes another monolith | Ownership changes only cosmetically | Restrict it to typed events, decisions, commands, and persistence; keep protocol/render mechanics in platform |
| Temporary PoC compatibility becomes permanent | Stage 3 generic host work is obscured | Isolate in `runtime/poc_compat.rs`, document deletion target, prohibit platform imports |
| Render extraction changes frame semantics | Overdraw, stalls, or buffer protocol errors | Move logic mechanically first; test state transitions and run redraw smoke before optimization |
| Compositor enrichment blocks event loop | Input/render stalls remain | Isolate current behavior now; schedule event-driven replacement after ownership extraction unless required for correctness |
| UWSM API is frozen from assumptions | Incorrect lifecycle/spawn contract | Define capability interface and injected detection only; validate concrete mechanics in Stage 7 |
| Large all-at-once move is hard to review | Hidden regressions and merge conflicts | Follow compile-safe sequence, one owner migration at a time, gates after each step |

## Explicit Stage 3+ Non-Goals

Stage 2 must not implement:

- generic `BarHost`, `FloatingHost`, `ClockWidget`, or `CalendarWidget` content
  separation beyond the generic native `SurfaceSpec` needed to keep platform
  free of widget kinds;
- replacement of Slint `Window`-bound Bar and Calendar UI;
- generic host/widget configuration or default-shell composition;
- continuous live drag, relative pointer integration, compositor movement
  adapters, movement capability UI, or drag cancellation semantics from Stage
  4;
- persistence topology completion beyond preserving current behavior and the
  listed resolver/readiness corrections;
- configuration loading/reload, themes, IPC, Rune, service expansion, launcher
  features, notification infrastructure, lock screen, or PAM;
- multiseat redesign, fractional scaling, reusable render buffers, or resource
  budget optimization;
- event-driven replacement of every compositor adapter unless extraction
  reveals a correctness blocker;
- full UWSM environment detection, lifecycle signaling, `uwsm app` mechanics,
  or managed-session acceptance testing;
- crate splitting or public extension APIs; or
- visual changes.

## Recommended Implementation-Agent Breakdown

Use one writer per ownership boundary. Later tasks begin only after their
dependencies are merged or handed off on a stable branch.

### Agent 1: Platform State Foundations

Implements Step 1 only: platform module root, topology generation, pure output
state, pure surface generation registry, typed commands/events, and tests. It
must not move raw Wayland code.

**Deliverable:** compiling vocabulary and exhaustive state-machine tests.

### Agent 2: Compositor and OutputManager Extraction

Starts after Agent 1. Implements Steps 2 and 3, including monotonic OutputIds,
canonical snapshot composition, geometry readiness, removal ordering, and
adapter enrichment isolation.

**Deliverable:** OutputManager owns inventory while the old runtime still
consumes its typed snapshot.

### Agent 3: Slint Render Bridge

Can begin design after Agent 1 but should integrate against a stable platform
root. Implements Step 4, including synchrony proof, HostId creation scope,
adapter registry, and move from `window.rs`.

**Deliverable:** no positional adapter association; visible output unchanged.

### Agent 4: SurfaceManager and Callback Generations

Starts after Agents 2 and 3. Implements Step 5, native routing, role lifecycle,
render subrecord integration, and stale callback tests.

**Deliverable:** HostId/generation-keyed native surfaces with controlled
cross-output recreation.

### Agent 5: Wayland Ownership Extraction

Starts after Agent 4. Implements Step 6: connection/global/dispatch/input
ownership, typed event delivery, calloop boundary, and raw-import cleanup.

**Deliverable:** all Wayland/SCTK objects under platform and passing smoke
startup/render/input.

### Agent 6: Coordinator Migration

Starts after Agent 5. Implements Step 7, including direct PlacementEngine use,
topology-generation reconciliation, geometry-ready candidates, persistence
triggers, HostId drag compatibility, and temporary PoC policy factory.

**Deliverable:** thin typed coordinator and removal of legacy pump/reconcile
ownership.

### Agent 7: Session Boundary

May proceed after platform root stabilizes but must avoid files actively owned
by Agents 4/5. Implements Step 8 with standalone default and injectable managed
selection only.

**Deliverable:** tested platform/session contract with no behavior dependency
on UWSM.

### Agent 8: Integration and Adversarial Verification

Does not author new architecture. Integrates Step 9, removes duplicate paths,
runs static boundary audits and all Rust gates, executes the Wayland smoke
matrix, and reports failures with exact evidence.

**Deliverable:** one clean ownership path, explicit smoke record, no Stage 3+
scope creep.

## Review Checklist

Before implementation approval, verify every task against these questions:

1. Does this move one existing responsibility to one named owner?
2. Can the step compile and be tested before the next move?
3. Does any new type duplicate domain or persistence truth?
4. Can a removed output or retired surface generation still reach an active
   decision or callback path?
5. Is HostId explicit at every Slint/native association boundary?
6. Does platform code remain generic with respect to Bar and Calendar?
7. Is observed PoC behavior preserved rather than redesigned?
8. Is UWSM optional and confined to platform/session?
9. Has the change avoided Stage 3+ work?
10. Are automated and manual claims clearly distinguished?

Stage 2 should finish with a smaller and more mechanical platform core, not a
larger framework. The extraction succeeds when protocol objects, topology,
native instances, rendering, logical decisions, and session integration each
have one clear owner while the existing shell remains demonstrably unchanged.
