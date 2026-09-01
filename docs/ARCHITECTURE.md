# Owlenshell Architecture

## Purpose

This document defines the production architecture of Owlenshell.

`IDENTITY.md` defines what Owlenshell is and what principles it follows. This document defines how those principles are represented in the runtime.

The architecture is informed by the atomic-surface PoC, but the production codebase must not preserve PoC shortcuts merely because they already work.

## Architectural Goals

The architecture must preserve:

- atomic Wayland surfaces
- compositor independence
- modern-native, self-contained Rust + Slint runtime
- direct manipulation as first-class desktop UX
- event-driven system integration
- persistent topology-aware placement
- strict separation of content, placement and system state
- unified typed configuration shared identically by file configs, Settings GUI, and IPC/agents
- human and agent operability without internal AI burdens or mutation backdoors
- failure isolation
- extensibility without making scripting part of the critical path
- observability, machine-readable schemas, and testability

## High-Level Model

```text
┌────────────────────────────────────────────────────────────┐
│                        Owlenshell                           │
│                                                            │
│   ┌────────────────────────────────────────────────────┐   │
│   │                   Shell Core                       │   │
│   │                                                    │   │
│   │ Widget Runtime    State Store      Event Bus       │   │
│   │ Config Runtime    Persistence      IPC             │   │
│   └───────────────────────┬────────────────────────────┘   │
│                           │                                │
│       ┌───────────────────┼────────────────────┐           │
│       ▼                   ▼                    ▼           │
│ Surface Runtime       System Services     Extensions       │
│       │                   │                    │           │
│       ▼                   ▼                    ▼           │
│ Placement/Input      DBus/PipeWire/...       Rune          │
│       │                                                    │
│       ▼                                                    │
│ Wayland Platform ───────────────► Compositor Adapters      │
│       │                                                    │
│       ▼                                                    │
│     Slint                                                  │
└────────────────────────────────────────────────────────────┘
```

## Ownership Boundaries

### Wayland Platform

Owns:

- Wayland connection
- registry/global lifecycle
- outputs
- seats
- pointer and keyboard events
- layer-shell roles
- xdg roles where required
- SHM/GPU rendering bridge
- scale/configure events
- frame callbacks
- protocol-specific lifecycle

Does not own:

- widget business logic
- persisted placement policy
- system services
- Rune scripts
- theme semantics

### Surface Runtime

Owns:

- creation and destruction of shell surfaces
- mapping a widget host to a native surface
- surface geometry
- input regions
- surface mode
- exclusive zone behavior
- visibility/lifetime policy
- recreation during output migration

A surface host must be able to preserve widget state across native-surface recreation.

### Placement Engine

Owns:

- saved placement intent
- runtime placement resolution
- output matching
- fallback
- return-home
- anchor/offset semantics
- clamping/reachability
- placement persistence triggers

The resolver should be mostly pure logic:

```rust
resolve(
    preference,
    widget_geometry,
    outputs,
    context,
) -> RuntimePlacement
```

Topology-driven fallback must never write user placement.

### Input Manager

Owns:

- widget movement
- resize interactions
- move handles
- edit/layout mode
- configurable bindings
- modifier state
- conflict reporting through compositor adapters
- pointer-grab state
- interaction cancellation on topology changes

The production implementation must support live visual movement while dragging.

### Widget Runtime

Owns:

- widget identity
- widget lifecycle
- host assignment
- binding typed state to Slint
- converting widget actions to runtime actions
- attaching/detaching content from hosts

A widget must not directly own its Wayland placement.

### State Store

Owns canonical typed runtime state.

Examples:

- audio state
- network state
- power state
- media state
- active workspace state
- notification state

State is produced by services and consumed by widgets, IPC and scripts.

### Event Bus

Owns typed runtime events and actions.

Avoid a global untyped string bus.

Events should have explicit schemas and ownership.

### System Services

Own native integrations such as:

- PipeWire
- NetworkManager
- BlueZ
- UPower
- MPRIS
- brightness/backlight
- notifications
- idle state
- clipboard backend
- compositor workspace/window state

Services are shared.

Two widgets consuming audio state must not create two independent PipeWire clients unless explicitly required.

### Slint UI

Owns:

- component layout
- presentation
- visual state
- transitions
- animation
- rendering inside an assigned surface

Slint must not become the owner of Wayland shell semantics.

### Rune Runtime

Owns optional user-defined behavior.

Rune may:

- subscribe to safe typed state
- transform values
- define actions
- implement custom lightweight widgets
- react to events
- provide formatting and automation

Rune must not:

- directly mutate arbitrary Slint internals
- bypass security boundaries
- own PAM authentication
- directly manage Wayland objects
- become required for built-in shell functionality

### Config Runtime

Owns declarative desired configuration.

Configuration describes what exists and how it is arranged.

Behavior belongs in Rune when logic is required.

Presentation belongs in styling.

### Persistence

Owns versioned machine-local user intent such as:

- widget placement
- selected mode
- size where applicable
- layout state

Persistence must distinguish user intent from runtime fallback.

### IPC

Owns external runtime control and inspection.

Expected future client:

```text
owlenshellctl
```

IPC must support inspection as a first-class operation.

### Compositor Adapters

Adapters enrich generic Wayland behavior.

Potential capabilities:

- focused output
- logical output role
- output serial/identity
- global output geometry
- reserved/work areas
- compositor binding introspection
- workspace/window events
- compositor-specific movement optimizations

Generic behavior must remain usable when an adapter is unavailable.

### Session and UWSM Integration

Owns session lifecycle detection, systemd user session cooperation, and session-aware application launching.

When running inside a UWSM-managed session:

- Detects UWSM environment variables and systemd user session state.
- Emits ready/stop notifications to cooperate with session lifecycle management.
- Delegates desktop application and action launches to `uwsm app -- ...` for clean cgroups and environment isolation.
- Avoids conflicting environment exports or duplicate session management.
- Degrades gracefully to standard POSIX process spawning when UWSM is absent.

All session and UWSM mechanisms live behind a platform integration trait and never leak into widget implementations or pure domain models.

## Widget vs Surface

A widget and an atomic surface are not identical concepts.

A bar is an atomic surface containing multiple widgets:

```text
BarSurface
├── WorkspaceWidget
├── WindowTitleWidget
├── ClockWidget
└── StatusWidget
```

A calendar may be independently hosted:

```text
CalendarWidget
      ↓
CalendarSurface
```

This distinction prevents the architecture from requiring one Wayland surface per tiny visual element.

## Hosting Model

A widget may move between hosts without changing its content implementation.

Possible hosts:

- bar container
- floating atomic surface
- popup atomic surface
- pinned atomic surface
- OSD surface
- notification host

Conceptually:

```text
Widget Content
      │
      ▼
 Widget Runtime
      │
      ▼
  Host Policy
      │
      ├── Bar
      ├── Popup
      ├── Floating
      └── Pinned
```

## Surface Modes

Built-in v1 modes:

### `cover`

Reserves desktop space where appropriate.

### `float`

Does not reserve desktop space.

Modes are policy bundles, not alternate widget implementations.

A mode may define:

- layer
- exclusive zone
- input behavior
- focus behavior
- dismissal behavior
- anchoring behavior

Custom user-defined modes should compile into validated policy objects.

## Placement Model

Persisted placement should be semantic:

```text
output preference
anchor
offset
constraints
mode
```

not desktop-global coordinates.

Output identity may contain:

- connector/name
- make
- model
- physical dimensions
- serial where available
- logical role where available

Transient Wayland global IDs must never be persisted.

## Output Resolution

Default resolution order:

1. exact configured connector/name
2. exact strong physical identity / serial
3. unique make/model/physical-size fingerprint
4. configured logical role
5. spawn/source output
6. compositor preferred/focused output
7. deterministic active output
8. defer if no valid output exists

Ambiguous identity matches must fall through.

Do not guess between indistinguishable physical monitors.

## Live Movement

Production Owlenshell must support live movement.

The visible widget should follow the pointer while dragging.

If generic Wayland cannot provide sufficient cross-output coordinates, compositor adapters may provide enhanced movement while the generic fallback remains functional.

Cross-output movement may require native-surface recreation.

Widget content/state must survive that recreation.

## Reachability

A persisted widget must never become permanently unreachable due to topology changes.

If a widget fits, clamp it to usable output bounds.

If it is larger than the output, preserve an accessible move/edit region.

If no valid output exists, defer the surface rather than inventing a position.

## Performance Model

Critical startup path:

```text
process start
→ load minimal config
→ connect Wayland
→ enumerate enough output state
→ create shell surfaces
→ first paint
→ shell ready
→ initialize optional/noncritical services
```

Optional service discovery must not block first paint.

Prefer event-driven updates.

Avoid subprocesses for state that can be read natively.

## Failure Model

Failures should remain local.

Examples:

- broken weather provider → weather widget degraded
- bad Rune script → script disabled, shell survives
- bad theme reload → old theme remains active
- missing compositor adapter → generic behavior
- network service unavailable → network widget degraded

Lock/auth failures are security-sensitive and require stronger fail-safe behavior.

## Security Boundaries

Security-sensitive paths must not share arbitrary script trust.

At minimum:

```text
ordinary widget/runtime domain
             │
             ├── safe typed auth request
             ▼
       authentication boundary
             │
             ▼
             PAM
```

Rune must never receive raw authentication secrets.

Lock failure must fail closed where appropriate.

## Dogfooding Rule

The shipped default shell must be constructed using the same public widget, placement, styling and host primitives available to users.

Do not build a private `DefaultBarSpecialCase` that users cannot reproduce.

## Evolution

Large architectural decisions must receive an ADR.

This document should remain stable at the subsystem level while implementation details evolve underneath it.
