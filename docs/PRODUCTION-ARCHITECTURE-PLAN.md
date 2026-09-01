# Production Architecture Plan

The atomic-surface proof of concept validates Owlenshell's core technical
choices, but its runtime structure should not become the production
architecture. Production v1 should remain one crate initially, split into
explicit Rust modules around a single-threaded shell runtime, generic hosts and
widgets, pure placement decisions, capability-aware input, and replaceable
Wayland and compositor infrastructure.

## Production Shape

```text
src/
  main.rs                 process bootstrap only
  app.rs                  composition root and event-loop coordination

  domain/
    ids.rs                HostId, WidgetId, OutputId, SeatId
    host.rs               HostSpec, HostMode, HostState
    widget.rs             WidgetSpec, WidgetState
    placement.rs          saved intent and resolved runtime placement
    output.rs             output identity, geometry, capabilities

  platform/
    wayland.rs            connection, registry, dispatch, protocol ownership
    output_manager.rs     live output inventory and topology generations
    input_manager.rs      seats, pointer/keyboard state, move sessions
    surface_manager.rs    native surface lifecycle and role recreation
    render_bridge.rs      Slint WindowAdapter and wl_shm rendering
    compositor.rs         adapter interface and capability discovery
    session.rs            session integration interface and capability discovery
    session/
      standalone.rs       standalone lifecycle and direct process execution
      uwsm.rs             optional UWSM session coordination and app execution

  runtime/
    coordinator.rs        applies domain decisions to platform objects
    host_runtime.rs       host/widget attachment and logical lifecycle
    placement_engine.rs   pure placement resolution
    state_store.rs        canonical typed runtime state
    event_bus.rs          typed internal events and commands

  persistence/
    repository.rs         load, migrate, validate, atomically commit
    schema.rs             versioned persisted representation

  widgets/
    clock.rs
    calendar.rs

  services/
    clock.rs

ui/
  primitives.slint
  bar_host.slint
  floating_host.slint
  clock_widget.slint
  calendar_widget.slint
```

These are module boundaries, not proposed crates. Split crates only when an
independently testable API, separate security domain, or external consumer
justifies it.

The runtime remains one thread initially because Slint's current software
window is `!Send`, and Wayland dispatch, surface mutation, and rendering already
have a coherent single-thread ownership model. Services may perform work
elsewhere later, but return typed messages to the shell thread.

## Ownership Model

### Application Coordinator

`App` is the composition root. It owns subsystem instances and drives one
ordered reconciliation loop:

1. Drain platform and service events.
2. Update output, input, host, widget, and canonical state.
3. Resolve affected placements.
4. Apply surface lifecycle commands.
5. Render dirty surfaces.
6. Persist only committed user intent.
7. Publish resulting typed events.

It must not contain calendar logic, geometry formulas, Wayland protocol calls,
or Slint component-specific branches.

### Wayland Platform

The Wayland platform exclusively owns:

- Wayland connection and queue
- Registry globals
- Output and seat protocol objects
- Layer-shell role objects
- Scale, configure, and frame lifecycle
- `wl_shm` resources
- Native input event translation
- Slint rendering bridge

Higher layers use typed commands and events. They do not retain raw Wayland
objects.

### Session and UWSM Integration

Owns session lifecycle cooperation and UWSM-aware process execution.

- **Optional Integration**: When running inside a UWSM-managed session (detected via environment variables such as `UWSM_APP_ID` / systemd user session markers), Owlenshell coordinates with session startup/shutdown lifecycles.
- **Application Execution**: The launcher and shell action runners spawn user applications via `uwsm app -- ...` when UWSM is active, ensuring proper cgroup assignment and environment inheritance.
- **Environment Isolation**: Owlenshell does not attempt duplicate or competing environment exports when UWSM is present.
- **Standalone Parity**: Standalone or non-UWSM launches continue to function without degradation using standard subprocess management.
- **Boundary Invariant**: All session/UWSM detection remains encapsulated behind a platform session integration interface; widgets and domain logic remain unaware of UWSM mechanics.

### Surface Manager

`SurfaceManager` owns native surface instances, not widget state. Its
conceptual API is:

```text
create(host_id, surface_spec, output_id)
reconfigure(host_id, geometry, input_region)
recreate_on_output(host_id, output_id)
show(host_id)
hide(host_id)
destroy(host_id)
request_frame(host_id)
```

Cross-output movement is a controlled role recreation:

1. Preserve `HostState` and attached `WidgetState`.
2. Retire the old layer surface.
3. Create the replacement against the target `wl_output`.
4. Rebind the same logical host.
5. Resume rendering after configure.

A stale callback from the retired generation must not mutate the replacement.
Every native instance therefore needs a monotonically increasing generation
token.

### Atomic Surface

`AtomicSurface` should be a runtime-owned record, not a public god object:

```text
AtomicSurface
  host_id
  native_generation
  native_lifecycle
  resolved_placement
  configured_geometry
  scale
  frame_state
  render_adapter
```

It represents one exact native interaction and rendering boundary. It does not
decide placement, own widget business state, calculate calendar data, or
persist itself.

### Host and Widget

A host owns native-shell policy. A widget owns content and behavior.

```text
BarHost
  mode: reserve
  geometry policy: top edge, output width, fixed height
  widgets: [ClockWidget]

FloatingHost
  mode: float
  placement: semantic saved intent
  movement policy: visible handle/edit mode
  widgets: [CalendarWidget]
```

`CalendarWidget` knows dates, month navigation, and calendar state. It does not
know margins, outputs, layer-shell anchors, drag coordinates, or persistence
paths.

Opening the calendar is a generic action:

```text
ClockWidget emits ToggleHost(calendar_host)
HostRuntime shows or hides FloatingHost(calendar_host)
```

There must be no `Calendar` variant in platform, surface, output, input,
placement, persistence, or rendering code.

## Core Domain Models

Use typed stable IDs instead of stringly coupled constants:

```text
HostId
WidgetId
OutputId
SeatId
```

`OutputId` is runtime identity for a currently advertised output. It must not
be persisted. Persisted output preference contains matching evidence:

```text
OutputPreference
  connector/name
  serial when available
  make/model/physical fingerprint
  logical role when configured
```

Placement is split explicitly:

```text
SavedPlacement
  preferred_output
  anchor
  offset
  mode

ResolvedPlacement
  runtime_output
  geometry
  resolution_reason
  is_fallback
  reachable
```

This prevents runtime fallback from overwriting user intent.

`HostSpec` carries generic policy:

```text
namespace
layer
mode
size policy
anchor policy
exclusive-zone policy
keyboard policy
input policy
movement policy
```

`cover` and `float` should be validated bundles that produce host policies,
not branches attached to widget types.

## Proof-of-Concept Classification

| PoC subsystem | Classification | Production decision |
|---|---|---|
| Custom Slint platform | **Keep concept** | Rust continues to own Slint platform integration and Wayland lifecycle. |
| Single-thread Slint/Wayland ownership | **Keep initially** | Matches current `!Send` renderer constraints and simplifies protocol ownership. |
| SCTK and layer-shell | **Keep** | Proven appropriate for bars, floating surfaces, OSDs, and detached calendar surfaces. |
| Exact visible/input surface geometry | **Keep invariant** | No invisible parent, oversized hitbox, or synthetic atomicity. |
| Frame callback throttling | **Keep, refactor** | Move into render bridge; preserve dirty and frame-pending state. |
| Per-frame fresh buffer allocation | **Replace** | Introduce reusable buffer and pool lifecycle behind render bridge. |
| `OwlenPlatform` adapter side channel | **Replace** | Use an explicit host-to-window creation handshake keyed by `HostId`; no positional pop or index association. |
| `SurfaceKind::{Bar, Calendar}` | **PoC-only** | Replace with `HostSpec`, `HostId`, and generic host runtime. |
| `AtomicSurface` current state embedded in monolith | **Refactor** | Make it a narrow runtime record owned by `SurfaceManager`. |
| `runtime.rs` monolith | **Replace** | Split by ownership: platform, outputs, surfaces, input, placement, rendering, and coordinator. |
| Existing output-removal ordering | **Keep invariant** | Evict removed outputs before placement reconciliation. |
| `OutputSnapshot` | **Refactor** | Separate protocol facts, enriched metadata, stable matching evidence, geometry, and capabilities. |
| Transient numeric output ID as fallback ordering | **Replace** | Deterministic fallback uses stable live properties, with runtime ID only as the final tie-breaker. |
| Saved intent versus runtime fallback | **Keep** | Make the distinction explicit in types and the persistence API. |
| Existing pure placement functions | **Keep and refactor** | Form the basis of `PlacementEngine`; expand inputs and result type. |
| Placement tied to bar output as spawn context | **Replace** | Spawn or source output is explicit context supplied by the initiating seat or host action. |
| Nearest-corner and clamp algorithms | **Keep** | Generalize to work area and host constraints. |
| Four-corner `Anchor` | **Refactor** | Support required edge and corner semantics without coupling to layer-shell flags. |
| Drop-only drag | **Replace** | Production v1 requires continuous visible movement. |
| Calendar-only drag detection | **PoC-only** | Movement starts from generic host movement policy and hit regions. |
| One global pointer and keyboard | **Replace** | `InputManager` tracks state per seat and device capability. |
| Surface-local fixed-origin drag arithmetic | **PoC-only for live motion** | Valid for release calculation only while the surface remains fixed; unsafe as a live movement source. |
| Window-bound `Bar` and `Calendar` Slint components | **Replace** | Separate host shells from widget content components. |
| Inline clock in bar | **PoC-only** | Implement `ClockWidget` attached to `BarHost`. |
| Calendar calculations in `main.rs` | **Replace** | Move to `CalendarWidget` and calendar domain logic. |
| Hardcoded sizes, namespaces, zones, and handles | **PoC-only** | Supply them through host and widget specs and defaults. |
| Placement TOML and temp-rename | **Keep primitive, refactor schema** | Use a general repository, typed IDs, migrations, validation, backup and recovery, and a durable write policy. |
| Silent reset on invalid persistence | **Replace** | Preserve the bad file for diagnosis and expose a typed recovery result. |
| Compositor adapter abstraction | **Keep concept, refactor interface** | Make it capability-oriented and event-driven, with clear provenance. |
| Synchronous `hyprctl` polling | **PoC-only, replace** | No subprocess in refresh or render paths; use event subscription or asynchronous cached integration. |
| Logging and instrumentation | **Refactor** | Use structured events keyed by host, surface generation, output, seat, and placement reason. |
| Clock timer in bootstrap | **PoC-only** | A native clock service publishes typed time updates. |
| State store | **Missing, add** | Typed canonical state for services, widgets, and runtime, independent from Slint properties. |
| Event bus | **Missing, add narrowly** | Typed enums or channels for cross-subsystem facts; direct calls remain preferable locally. |
| Config loader | **Missing, stage later** | Transactional desired-state input; defaults must use the same schema. |
| IPC | **Missing, defer** | Add only after runtime commands and queries stabilize. |
| Rune | **Missing, defer** | Bind to typed state and actions, never Wayland or Slint internals. |
| Lock/PAM | **Outside initial runtime** | Use a separate protected boundary; fail closed and never expose credentials to Rune or widgets. |

## Placement Engine

`PlacementEngine` should be mostly pure:

```text
resolve(saved, outputs, context, host_constraints) -> PlacementDecision
commit_drop(pointer, host_geometry, outputs, constraints) -> SavedPlacement
```

Resolution order:

1. Exact configured output name or connector
2. Strong serial or physical identity
3. Unique hardware fingerprint
4. Configured logical role
5. Spawn or source output
6. Adapter-reported focused or preferred output
7. Deterministically ordered active output
8. Deferred when none are reachable

Ambiguous matches fall through. They are not guessed.

`PlacementDecision` records the selected output, geometry, fallback status,
matching reason, and diagnostics. Runtime reconciliation reacts to this
decision; the engine never creates surfaces or writes files.

Return-home behavior follows automatically:

- Preferred output present: resolve home.
- Preferred output absent: resolve fallback without mutating `SavedPlacement`.
- Preferred output returns: the next topology reconciliation resolves home and
  recreates the surface there.
- No outputs: the host remains logically alive but its native surface is
  deferred.

Persistence occurs only after a committed user move or explicit configuration
change.

## Production Live Drag

The PoC's surface-relative pointer events cannot safely drive live movement
while the surface itself moves. Once margins change, the coordinate frame
changes and deltas become invalid. A visible handle solves initiation, not
coordinate acquisition.

Production v1 therefore needs an explicit movement capability:

```text
MovementCapability
  global_pointer_stream
  stable_relative_motion
  compositor_move_session
  unavailable
```

Preferred implementation:

1. A pointer press on a host-declared movement region starts `MoveSession`.
2. The input adapter captures a stable global or relative motion stream.
3. Each motion event computes candidate global geometry.
4. The placement engine clamps it against the current output or work area.
5. The surface manager updates the visible surface before the next frame.
6. Crossing an output boundary triggers controlled surface recreation while
   the logical move session continues.
7. Release converts final geometry to semantic output, anchor, and offset.
8. Persistence writes the new saved intent once.
9. Escape, seat loss, output loss, or protocol failure restores the pre-drag
   semantic placement.

Do not advertise draggable movement when capability is `unavailable`.
Drop-only relocation does not satisfy v1 and should not be retained as a
misleading generic fallback.

This creates a product gate: production v1 requires live movement on at least
the explicitly supported primary compositor path. Generic Wayland may still
provide all non-movement shell behavior, but must report movement capability
honestly. Broader compositor support follows through adapters after each
coordinate source is validated.

## State and Events

`StateStore` contains canonical typed values such as time, calendar view state,
host visibility, and service status. Slint receives projections of that state.

The event bus should be schema-owned and small:

```text
PlatformEvent
OutputEvent
InputEvent
HostCommand
HostEvent
WidgetAction
StateChanged
PersistenceEvent
```

Do not turn every function call into an event. Use events where ownership
boundaries or asynchronous producers require them.

Services are native Rust providers. The first slice needs only `ClockService`.
Calendar arithmetic is widget or domain logic, not a system service.

## Deferred Boundaries

### Configuration and Graphical Settings UI

Do not begin by designing a complete TOML language. First represent defaults as
typed `AppSpec` data. Once the generic calendar slice works, add:

```text
Built-in defaults
        \
File configuration
         > Typed Configuration Model
Settings UI
        /
         -> validation / transactional configuration update
         -> runtime reconciliation
         -> HostSpec / WidgetSpec composition
```

Owlenshell ships a batteries-included native graphical settings UI for common
shell customization (e.g. bar position, bar size, output preferences, widget
enable/disable, widget ordering, common widget options, and later theme/styling).

Key architectural invariants:
- **Unified Public Model**: The settings UI is NOT a separate configuration
  system. Both GUI and file configuration consume and produce the exact same
  typed public configuration model. There is zero GUI-only runtime mutation or
  divergent configuration path.
- **Transactional Safety**: Invalid changes (from file edits or UI actions) are
  rejected during validation and never destroy the currently valid running
  composition.
- **Live Reconciliation**: Settings updates reconcile live without process restart
  where practical.
- **Non-Mandatory GUI**: The graphical settings UI must never become required for
  declarative, headless, or manual file-based configuration.
- **Revert to Defaults**: The system natively supports resetting or reverting to
  useful built-in defaults.
- **Rune Separation**: Rune remains an isolated behavioral/scripting extension
  mechanism, distinct from core configuration.

An invalid reload keeps the previous valid runtime.

### Persistence

Persistence stores machine-local runtime intent, not desired configuration.
The initial schema needs host placements and small widget state only. It must
be versioned from the first production write.

### IPC

Add IPC after `HostCommand` and state queries are stable. IPC should translate
external requests into the same commands used internally.

### Rune

Add Rune after state, actions, and configuration reconciliation are stable.
Rune receives typed snapshots and action handles. It gets no raw Wayland
objects, Slint handles, arbitrary internal event injection, or authentication
data.

### Lock and PAM

Treat lock and authentication as a later protected subsystem with its own
process or strongly isolated module boundary. Do not route PAM conversations
through the general event bus, Rune, widget state, logs, or ordinary IPC.

## First Production Vertical Slice

The first slice proves only this composition:

```text
BarHost
  ClockWidget

FloatingHost
  CalendarWidget
```

Required flow:

1. Defaults instantiate both hosts and widgets through generic specs.
2. `BarHost` creates an exact layer surface and reserves its configured zone.
3. `ClockWidget` receives typed updates from `ClockService`.
4. Activating the clock emits `ToggleHost(calendar_host)`.
5. `FloatingHost` hosts `CalendarWidget`; the runtime sees only host and widget
   IDs.
6. Calendar state survives hide and show and surface recreation.
7. The user drags the floating host with continuous visual movement.
8. Drop persists semantic output, anchor, and offset.
9. Restart restores saved intent.
10. Output removal moves the host to a runtime fallback without persistence.
11. Output return moves it back home automatically.
12. No output defers the native surface while preserving logical state.

A repository-wide search for calendar-specific platform behavior should find
nothing outside `widgets/calendar`, Slint calendar content, composition or
default data, and focused tests.

## Staged Delivery

### Stage 0: Freeze Contracts

Define typed domain models and subsystem interfaces before moving behavior.

Acceptance gates:

- Saved and resolved placement cannot be confused by type.
- Host and widget IDs are stable and typed.
- Host policy contains no widget-kind branches.
- Movement capability and degradation are explicit.
- No new crate split.

### Stage 1: Extract Pure Core

Move and expand placement and output matching into pure domain and runtime
modules. Introduce host and widget specs and a versioned persistence
representation.

Acceptance gates:

- Existing placement scenarios remain covered.
- Tests cover ambiguity, stable fallback ordering, no-output defer, fallback
  without persistence, return-home, clamp, and semantic drop.
- Persistence tests cover current version, unsupported version, corruption
  preservation, and atomic replacement failure.
- Pure tests require no Wayland connection.

### Stage 2: Establish Platform Ownership

Separate output, surface, rendering, and coordinator ownership while retaining
the currently proven visible behavior.

Acceptance gates:

- Raw Wayland objects exist only under `platform`.
- Surface recreation is keyed by `HostId` and generation.
- Removed outputs cannot win resolution or receive stale callbacks.
- Slint adapter association uses explicit identity, not queue position.
- Session integration isolates standalone process spawning from optional UWSM coordination behind platform boundaries.
- Rendering remains frame-throttled.
- Calendar and bar branches are absent from platform modules.

### Stage 3: Separate Hosts and Widgets

Implement generic `BarHost`, `FloatingHost`, `ClockWidget`, and
`CalendarWidget`.

Acceptance gates:

- Clock and calendar content can be instantiated independently of native
  surface policy.
- Toggle uses a generic host command.
- Calendar widget state survives hide and show and native recreation.
- The default shell is built from public specs, not private bootstrap branches.

### Stage 4: Prove Live Movement

Implement and validate the compositor capability needed for stable movement.

Acceptance gates:

- The visible floating host follows every accepted motion update.
- No cumulative coordinate drift occurs while the surface moves.
- Same-output and cross-output drag both work.
- Cross-output recreation does not reset calendar state.
- Release persists once; intermediate motion never persists.
- Cancel restores the original placement.
- Output or seat loss during drag leaves the host reachable or safely deferred.
- Unsupported environments report movement unavailable rather than using
  drop-only behavior.

This is the highest-risk and earliest required spike. It should happen before
configuration, Rune, IPC, or service expansion.

### Stage 5: Complete Persistence and Topology Flow

Wire restart restoration, fallback, and return-home through generic placement
reconciliation.

Acceptance gates:

- Preferred output loss does not modify persisted intent.
- Restart during fallback still preserves the original home preference.
- A returning output moves the host home.
- Ambiguous identities never cause arbitrary migration.
- Zero-output startup and reconnect preserve logical host and widget state.

### Stage 6: Add Typed Default Configuration and Transactional Loading

Represent the dogfood shell through typed defaults, then add transactional file
loading. The typed configuration model must be designed from the start to serve
both declarative file configuration and the graphical settings UI.

Acceptance gates:

- Missing configuration produces the same useful default composition.
- Invalid configuration leaves the running composition unchanged.
- Host and widget add, update, and removal reconcile without process restart.
- Persisted runtime intent and declarative configuration remain separate.

### Stage 6b: Native Graphical Settings UI

Build the batteries-included graphical settings UI on top of the Stage 6 typed
configuration model.

Acceptance gates:

- Settings UI directly reads and modifies the shared typed configuration model.
- Common settings (bar position/size, output preference, widget enable/disable/ordering)
  can be configured graphically.
- Settings apply live without process restart where practical.
- Reset/revert to built-in defaults is supported.
- Zero divergent or GUI-only configuration state.
- Declarative file configuration remains fully supported and never requires the GUI.

### Stage 7: Harden the Slice

Address buffers, scaling policy, multiseat input, diagnostics, and failure
recovery.

Acceptance gates:

- Reusable render buffers avoid steady per-frame allocations.
- Integer scaling and output changes are tested.
- Fractional scaling is either implemented and tested or explicitly rejected.
- Each seat has independent focus, modifiers, and move session state.
- Lifecycle traces identify host, output, seat, generation, and resolution
  reason.
- Session integration is verified in both standalone and UWSM-managed environments (cooperative lifecycle signaling and application execution delegation).
- Idle and active resource budgets are measured, not inferred.

### Later Stages

Only after the vertical-slice gates pass:

1. Shared services beyond the clock
2. Inspect and control IPC
3. Rune behavior runtime
4. Broader compositor adapters
5. Protected lock and PAM subsystem
6. Possible crate extraction based on proven boundaries

## Principal Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| No stable live-drag coordinate source | The v1 movement requirement cannot be met generically. | Validate one primary compositor capability during Stage 4 before expanding scope. |
| Cross-output recreation during drag | Flicker, lost input session, or stale callbacks. | Keep the logical move session independent of native surface generation and use an explicit recreation handshake. |
| Output identity ambiguity | A surface migrates to the wrong monitor. | Use evidence-ranked matching and ambiguity fallthrough, and never persist fallback. |
| Slint adapter association race | The wrong UI binds to a native host. | Use an explicit `HostId` creation token and remove the positional side channel. |
| Runtime coordinator becomes another monolith | Boundaries exist only in filenames. | Enforce exclusive ownership and pure decision APIs; test placement without the platform. |
| Event bus becomes untyped global coupling | Control flow becomes hidden. | Use small schema-owned enums and direct calls within a subsystem. |
| Configuration is designed too early | The public model freezes prematurely. | Build typed defaults first and serialize only after the vertical slice stabilizes. |
| Persistence schema leaks runtime objects | Migration and recovery become fragile. | Use dedicated persisted DTOs converted to domain types. |
| Compositor adapter blocks the event loop | Input and rendering stall. | Use an event-driven or asynchronous adapter feeding a cached capability snapshot. |
| Lock or authentication shares ordinary extension trust | Credentials or unlock flow can be compromised. | Use a separate protected boundary with no Rune or general-bus access. |
| Documentation overpromises portable live movement | Unsupported compositors appear broken. | Publish a capability matrix based only on exercised adapters. |

## Recommendation

Start with Stages 0 and 1, but treat Stage 4's live-drag capability as an early
feasibility gate rather than late polish. The remainder of the architecture is
supported by the PoC. Stable continuous movement across native layer surfaces
is the unresolved mechanism most capable of invalidating the v1 interaction
contract.

The production architecture should preserve what the PoC proved, not its
object structure: custom Rust-owned Wayland and Slint integration, exact layer
surfaces, logical-state-preserving recreation, semantic placement, and
fallback and return-home. Everything calendar-specific in the current runtime
is scaffolding to remove.
