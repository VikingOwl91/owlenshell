# Owlenshell

> A self-contained, native Wayland desktop shell built around atomic, movable and persistent UI surfaces.

## Identity

Owlenshell is a native Wayland desktop shell.

It is not a window manager and it is not a compositor.

The compositor owns windows, workspaces, composition and global input policy.

Owlenshell owns the desktop shell around them:

- bars
- widgets
- notifications
- OSDs
- clipboard UI
- launcher
- idle handling
- lock screen
- authentication integration
- shell-level desktop UI

The long-term goal is to provide a complete daily-driver desktop shell that can run on different Wayland compositors while maintaining the same shell experience.

```text
┌─────────────────────────────────────────────┐
│                 Owlenshell                  │
│                                             │
│   Bar     Notifications     OSD             │
│   Widgets Clipboard         Launcher        │
│   Lock    Idle              Shell UI        │
│                                             │
│        Atomic Surface Runtime               │
│   Placement · Input · State · Services      │
├─────────────────────────────────────────────┤
│               Wayland APIs                  │
├─────────────────────────────────────────────┤
│ Hyprland │ niri │ Sway │ Mango │ ...        │
└─────────────────────────────────────────────┘
```

## Core Principle: Atomic UI

Atomic UI is the defining architectural principle of Owlenshell.

An atomic widget is not merely a rectangle rendered inside a larger shell window.

It is an independently hosted desktop UI unit with its own:

- Wayland surface
- visible geometry
- input region
- lifecycle
- placement policy
- output assignment
- persisted placement
- state connection

This leads to two fundamental rules:

> **Atomic is architectural, not merely visual.**

> **No invisible interaction geometry.**

If a widget is visually 300×400 pixels, Owlenshell should not create an invisible output-sized interaction surface behind it.

Invisible widgets should not leave invisible input geometry behind.

Atomic widgets should behave like independent desktop objects.

## Movable by Design

The user owns the layout.

Atomic widgets can be moved independently and their placement can persist across restarts.

Moving a widget should be live:

```text
pointer moves
     ↓
widget moves
     ↓
surface visually follows
     ↓
drop
     ↓
persist placement
```

A widget should not remain at its old location during a drag and teleport only after the pointer is released.

Placement is semantic rather than based on permanent desktop-global coordinates.

Conceptually:

```text
Widget
  ↓
preferred output
  ↓
anchor
  ↓
offset
  ↓
placement policy
```

rather than:

```text
x = 3847
y = 281
```

The runtime must distinguish between user intent and temporary runtime decisions.

## Placement

Owlenshell provides four standard placement directions:

- `top`
- `bottom`
- `left`
- `right`

and supports user-defined custom placement.

Placement behavior is implemented as policy rather than hardcoded widget-specific behavior.

Widgets should not need to know where they are hosted.

The same widget should be capable of being hosted in a bar, popup, floating surface or another compatible shell container without rewriting its internal logic.

## Surface Modes

Owlenshell provides at least two standard surface modes.

### Cover

A cover surface intentionally occupies desktop space.

Typical example:

- a desktop bar using an exclusive zone

### Float

A floating surface does not reserve desktop space.

Typical examples:

- calendar
- volume popup
- clipboard
- launcher
- floating widgets

The underlying model should support user-defined modes later without accumulating special-case behavior throughout the surface runtime.

Surface modes are policies.

They are not separate widget implementations.

## Topology-Aware by Default

Multi-monitor behavior is a core runtime responsibility.

Outputs can:

- appear
- disappear
- change scale
- rotate
- be renamed
- temporarily become unavailable
- have identical make/model information
- return after a temporary disconnect

Owlenshell therefore distinguishes between:

```text
SavedPlacement
```

and:

```text
RuntimePlacement
```

The saved placement represents user intent.

The runtime placement represents where the widget can currently exist.

For example:

```text
preferred: HDMI-A-1

HDMI unavailable
       ↓
temporary fallback → DP-1

HDMI returns
       ↓
return → HDMI-A-1
```

Temporary topology fallback must never silently overwrite persisted user intent.

When the preferred output becomes available again, the widget should automatically return when appropriate.

Output matching must avoid relying on transient Wayland global IDs.

## Wayland Native

Owlenshell is designed specifically for Wayland.

Wayland is not treated as an implementation detail hidden behind a traditional desktop toolkit.

Owlenshell directly owns the Wayland semantics required by the shell.

The architecture follows:

> **Generic Wayland first, compositor enhancement second.**

The core must not require Hyprland, Sway, niri or another specific compositor.

Generic Wayland behavior provides the baseline.

Compositor adapters may enhance it.

```text
              Owlenshell Core
                     │
          Generic Wayland Backend
                     │
        ┌────────────┼────────────┐
        │            │            │
    Hyprland       niri         Sway
     Adapter       Adapter      Adapter
```

Adapters may provide additional information such as:

- focused output
- stronger output identity
- reserved geometry
- compositor-specific events
- binding introspection
- additional placement information

Missing compositor-specific functionality should cause graceful degradation rather than breaking the shell.

## Session Management and UWSM Integration

Owlenshell natively supports Universal Wayland Session Manager (UWSM) when running inside a UWSM-managed session.

UWSM support is first-class but strictly optional:

- **Optional Baseline**: Owlenshell remains fully functional without UWSM in standalone or compositor-launched environments.
- **Session Lifecycle Cooperation**: When UWSM is active, Owlenshell detects the environment, coordinates with the surrounding systemd user session lifecycle, and cooperates on startup/shutdown signals.
- **Clean Application Spawning**: Launcher and action subsystems use UWSM application invocation (`uwsm app -- ...`) where available to ensure clean cgroup management and environment propagation without leaking parent process state.
- **No Duplicate Management**: Owlenshell avoids conflicting or redundant environment manipulation when UWSM is already governing the session environment.
- **Strict Boundary**: All UWSM-specific detection and mechanics remain isolated behind a platform/session integration boundary and never leak into widgets or pure domain logic.

## Self-Contained

Owlenshell should not depend on GTK, Qt, GJS, QML or a browser runtime as its desktop shell foundation.

The intended stack is approximately:

```text
Rust
├── Wayland
├── system integration
├── state
├── services
├── persistence
└── Slint
      ↓
   rendering
```

Rust owns runtime behavior and system integration.

Slint owns presentation and rendering.

Slint is the UI layer.

It is not the desktop runtime.

Wayland surface semantics, placement, lifecycle, persistence and system state belong to Owlenshell.

## Performance Is Architectural

Performance is not merely a benchmark target.

It is an architectural constraint.

When nothing happens, Owlenshell should do almost nothing.

Prefer:

```text
Wayland events
DBus signals
PipeWire events
netlink
inotify
native system events
```

over:

```text
every N milliseconds:
    spawn process
    parse stdout
    update widget
```

System state should normally be collected by native shared services.

Widgets subscribe to typed state and receive updates when state changes.

Multiple widgets requiring the same information should not independently poll the same system resource.

The general performance goals are:

- fast startup
- near-zero idle CPU usage
- low memory overhead
- minimal unnecessary wakeups
- minimal subprocess usage
- no blocking service discovery on the critical first-paint path

Concrete performance budgets should be established using the production daily-driver configuration rather than treating early PoC measurements as guarantees.

## Batteries Included

Owlenshell is a desktop product, not merely a toolkit for building one.

A user should be able to install Owlenshell and receive a useful desktop shell without writing scripts or assembling every component manually.

The default experience should include common widgets such as:

- workspaces
- clock
- calendar
- active window/title
- system tray
- audio
- microphone
- media/MPRIS
- network
- Bluetooth
- battery
- power state
- brightness
- basic system information
- power/session actions

The shell itself should provide:

- bar
- notifications
- OSD
- clipboard UI
- launcher
- idle handling
- lock screen
- authentication integration

Customization extends the default experience.

Customization is not required to obtain a usable shell.

> **Customization is additive, not mandatory.**

## Configuration

Owlenshell should provide sensible defaults.

A basic installation should work without requiring the user to first design their own desktop.

At the same time, advanced users should be able to customize:

- widgets
- layout
- placement
- bindings
- surface modes
- styles
- actions
- scripts
- shell behavior

The intended responsibility boundaries are:

```text
config
  ↓
WHAT exists

Rune
  ↓
WHAT it does

CSS / SCSS
  ↓
HOW it looks

Slint
  ↓
HOW it is rendered

Rust
  ↓
WHAT is true
```

These boundaries should remain explicit.

## Styling

Slint is the rendering layer, but user styling should not require users to write Slint UI code.

The intended direction is a restricted styling frontend, potentially supporting CSS/SCSS-like input:

```text
SCSS / CSS
     ↓
 parser/compiler
     ↓
   StyleIR
     ↓
validated theme
     ↓
Slint properties/globals
```

Owlenshell should not attempt to implement a web browser's complete CSS cascade and layout engine.

The styling language exists to describe Owlenshell presentation.

Themes should support hot reload.

Invalid new styling should not destroy the currently active valid theme.

## Extension Layer

Native built-in functionality remains implemented in Rust.

Rune provides an optional extension layer for functionality such as:

- custom widgets
- formatting
- actions
- hooks
- event reactions
- lightweight automation
- user-defined behavior

Rune must not become the critical runtime path.

Scripts should interact with typed Owlenshell state and actions rather than directly mutating arbitrary Slint internals.

Conceptually:

```text
Rune
 ↓
typed state / action
 ↓
validation
 ↓
Rust runtime
 ↓
Slint
```

A broken custom script must not take down unrelated shell functionality.

## Failure Isolation

The shell should degrade locally whenever possible.

A broken:

- custom widget
- script
- system service
- optional backend
- theme reload
- compositor integration

must not unnecessarily terminate the entire desktop shell.

Security-sensitive functionality requires stronger isolation.

In particular:

- lock
- authentication
- PAM integration

must not share the same trust assumptions as arbitrary user widgets or scripts.

A broken weather widget must never compromise authentication or unlocking behavior.

## Input and Edit Mode

Movable atomic widgets require first-class input management.

Owlenshell should support configurable actions for operations such as:

- move
- resize
- pin
- unpin
- open
- close
- toggle
- change mode
- change output

Where supported, compositor adapters may detect conflicts with compositor-level bindings.

Explicit user configuration must not be silently rewritten.

Owlenshell should also provide a portable interaction mechanism that does not depend on compositor-global modifier introspection.

A dedicated layout/edit mode is the preferred long-term direction.

Normal mode operates the desktop.

Edit mode edits the desktop.

## Persistence

Persistent state must distinguish between:

- user intent
- runtime state
- temporary topology decisions

User-selected placement should survive restart.

Temporary output fallback should not overwrite it.

Persistence formats should be:

- versioned
- machine-local where appropriate
- atomically written
- migration-capable

Configuration and persisted state should be evolvable without requiring users to regularly delete their configuration directory.

## Hot Reload

Where practical, Owlenshell should support reloading:

- configuration
- styling
- Rune scripts
- widget layouts

without restarting the complete shell.

Reloads should be transactional where possible.

If a new configuration or theme is invalid, Owlenshell should retain the previous valid state and report the error.

## IPC and Diagnostics

Owlenshell should expose a native IPC interface and command-line control tool.

A future CLI may use the name:

```text
owlenshellctl
```

Typical operations should include:

```text
widget open
widget close
widget toggle

reload
layout edit
profile switch

inspect surface
inspect output
dump state

doctor outputs
doctor bindings
doctor placement
```

Diagnostics are a product feature.

Wayland shell problems should not require attaching a debugger merely to discover where the runtime believes a widget is located.

## Accessibility

Owlenshell should not make mouse gestures the only way to access important functionality.

Where technically possible, shell UI should support:

- keyboard navigation
- visible focus
- accessible actions
- non-drag alternatives to drag interactions
- visible move/edit handles
- assistive technology integration

Accessibility constraints should be considered while designing the widget and input APIs rather than added after those APIs become stable.

## Owlenshell v1

Owlenshell v1 is intended to be a complete daily-driver Wayland desktop shell based on independently hosted atomic UI surfaces.

The v1 target includes:

- atomic Wayland surface runtime
- movable widgets
- live movement while dragging
- persistent semantic placement
- multi-output topology handling
- fallback and automatic return-home
- standard placement policies
- `cover` and `float` surface modes
- user-defined/custom placement and mode capabilities
- bar
- standard daily-driver widgets
- notifications
- OSD
- clipboard UI
- launcher
- idle handling
- lock screen
- PAM/authentication integration
- configuration
- styling
- extension mechanism
- hot reload where practical
- IPC
- diagnostics

The exact number of built-in widgets is not itself the definition of v1.

The practical acceptance criterion is:

> **Owlenshell v1 is ready when it can replace a real daily-driver Wayland shell without losing the functionality that actually matters during daily use.**

## Explicit Non-Goals for v1

Owlenshell v1 does not need to:

- become a Wayland compositor
- become a window manager
- own application windows
- become a complete desktop environment
- perfectly support every Wayland compositor
- include a display manager/greeter
- absorb every related Owl project
- expose every internal implementation detail to scripts

Most importantly:

> **Owlenshell does not own your windows.**

That remains the compositor's responsibility.

## Greeter

A greeter/display-manager frontend is a possible future extension, but it is not a v1 requirement.

Owlenshell v1 should avoid architectural decisions that unnecessarily prevent a future greeter.

However, greeter functionality introduces additional concerns around:

- session creation
- privileges
- authentication
- failure recovery
- display-manager integration

It should therefore be treated as its own security-sensitive project milestone rather than "another widget".

## Launcher and Owlry

Owlenshell v1 requires a launcher.

Owlry may eventually provide the native backend for that launcher.

The architecture should allow a future separation such as:

```text
             Owlry Core
            /         \
           /           \
 standalone         Owlenshell
  launcher           launcher UI
```

Reusable functionality may include:

- providers
- indexing
- search
- ranking
- application state
- launch actions
- custom actions

Owlenshell can provide:

- surface hosting
- layout
- styling
- interaction
- desktop integration

Owlry should not be prematurely absorbed solely because Owlenshell requires launcher functionality.

## Long-Term Direction

The original problem may begin with a better bar.

The intended destination is larger:

> **A complete native Wayland shell whose desktop UI consists of independent atomic desktop objects.**

Conceptually:

```text
                 Wayland Compositor
                        │
                   Owlenshell
                        │
       ┌────────────────┼────────────────┐
       │                │                │
     Desktop          System          Extension
       │                │                │
      Bar             Audio            Rune
      OSD             Network          Plugins
      Notify          Bluetooth        Actions
      Clipboard       Power            Custom UI
      Launcher        Media
      Lock            Idle
       │
       └────────────┬───────────────────┘
                    │
             Atomic UI Runtime
                    │
             Rust + Slint + Wayland
```

The shell should remain useful at both extremes.

For someone who wants a complete desktop:

```text
install
   ↓
run
   ↓
usable desktop shell
```

For someone who wants to build their own environment:

```text
Owlenshell
├── config
├── styles
├── custom layouts
├── Rune
├── actions
├── custom widgets
├── custom placement
└── custom surface behavior
```

The guiding product principle is:

> **Owlenshell should feel like a desktop product when installed, but behave like a programmable shell when customized.**

A concise product expression is:

> **Here is your desktop. Make it yours.**

The design philosophy is opinionated without being restrictive:

- sensible by default
- powerful by design
- never restrictive by accident

## Architectural North Stars

When architectural decisions are ambiguous, prefer the option that preserves these properties:

1. **Atomic** — UI surfaces are independent desktop objects.
2. **Native** — Wayland and system integration are owned directly by Owlenshell.
3. **Self-contained** — No GTK, Qt, GJS, QML or browser runtime as the shell foundation.
4. **Compositor-agnostic** — Generic Wayland provides the baseline; compositor adapters enhance it.
5. **Movable** — The user owns widget placement and can manipulate it directly.
6. **Persistent** — User intent survives restarts and temporary topology changes.
7. **Topology-aware** — Multi-output behavior is part of the core architecture.
8. **Event-driven** — Idle systems should remain idle.
9. **Batteries-included** — A useful shell exists before customization begins.
10. **Programmable** — Advanced users can extend behavior without replacing the runtime.
11. **Fault-tolerant** — Optional components fail locally whenever possible.
12. **Secure by boundary** — Authentication and lock functionality are isolated from ordinary extensibility.
13. **Observable** — Runtime decisions can be inspected and diagnosed.
14. **Evolvable** — Configuration, state and APIs can migrate without repeatedly breaking user environments.
15. **Dogfooded** — The default shell must use the same public primitives available to users.

## The Question Owlenshell Should Keep Answering

The initial PoC answered:

> **Can native atomic Wayland widgets work?**

The production architecture must answer:

> **Can we turn that model into a complete desktop shell without sacrificing its atomicity, performance, portability or simplicity?**

And the final product should answer:

> **Can this replace the shell I actually use every day?**
