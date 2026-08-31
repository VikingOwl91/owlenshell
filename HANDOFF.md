# Project Handoff: Owlenbar

## Goal

Build a native Wayland shell/bar runtime focused on:

- Rust-first architecture
- Slint for UI/rendering
- Rune for optional scripting and user logic
- atomic Wayland surfaces
- compositor-independent design wherever possible
- persistent, topology-aware widget placement
- very low idle overhead
- fast startup
- no GTK / Qt / GJS / QML dependency stack

Working name:

`owlenbar`

The long-term goal is explicitly **not** merely "another Waybar clone".

The interesting part is a shell runtime where widgets are independently placeable UI units that can exist as:

- bar children
- ephemeral popups
- pinned widgets
- floating widgets
- OSDs
- notifications

without changing their actual content component.

---

# Core Design Principles

## 1. Atomic Widgets

A widget should only own the pixels it visibly occupies.

Avoid designs such as:

```text
full-height transparent layer surface
└── visible popup in the middle
```

Prefer:

```text
bar surface

audio popup surface

calendar popup surface

notification surface

osd surface
```

Each interactive popup/widget should ideally have its own actual Wayland surface with geometry matching its visible UI.

Hard invariant:

> No invisible interaction geometry.

Transparent unused regions must not intercept pointer input.

---

## 2. Content != Placement

Widget content must not know where it lives.

Example:

```text
AudioWidget
```

may be hosted as:

```text
bar child
popup
floating widget
pinned widget
```

without implementing multiple `AudioWidget` versions.

Conceptually:

```text
CONTENT
What is this?
audio / weather / calendar / metrics

PRESENTATION
How does it look?
Slint

PLACEMENT
Where/how does it exist?
bar / popup / floating / pinned / OSD
```

These concerns should remain separated.

---

## 3. Rust Owns System State

System integration should live in native Rust services.

Examples:

```text
AudioService
NetworkService
BluetoothService
MprisService
PowerService
NotificationService
TrayService
BrightnessService
OutputService
CompositorService
```

Widgets consume typed state.

For example:

```rust
AudioState
AudioEvent
AudioAction
```

The `AudioWidget` must not contain PipeWire integration directly.

The same `AudioState` should be usable simultaneously by:

- bar indicator
- volume OSD
- large mixer popup
- Rune scripts

---

# Proposed Architecture

```text
┌──────────────────────────────────────────────┐
│                  Slint UI                    │
│ widgets / layouts / animations / themes     │
└───────────────────────┬──────────────────────┘
                        │ typed state/actions
┌───────────────────────▼──────────────────────┐
│                Shell Runtime                 │
│                                              │
│ WidgetManager        SurfaceManager          │
│ PlacementResolver    OutputManager           │
│ StateStore           EventBus                │
│ RuneRuntime          IPC                     │
└───────────────────────┬──────────────────────┘
                        │
┌───────────────────────▼──────────────────────┐
│               System Services                │
│                                              │
│ Audio   Network   Bluetooth   MPRIS          │
│ Tray    Power     Notifications              │
│ Brightness   Idle   etc.                     │
└───────────────────────┬──────────────────────┘
                        │
┌───────────────────────▼──────────────────────┐
│             Platform / Wayland               │
│                                              │
│ Outputs / Seats / Surfaces                   │
│ layer-shell / xdg / popup                    │
│ compositor adapters                         │
└──────────────────────────────────────────────┘
```

---

# UI

Use Slint.

Slint should own:

- visual component tree
- layout
- animation
- widget state presentation
- hover/active/urgent states

Rust should pass typed data into Slint.

Do not let arbitrary scripts directly manipulate Slint widgets.

Avoid:

```text
Rune
-> find_widget("foo")
-> mutate arbitrary properties
```

Prefer:

```text
Rune
-> typed action/state
-> Rust validates
-> StateStore
-> Slint renders
```

---

# Scripting

Use Rune as an optional scripting layer.

Rune should be for:

- custom modules
- event hooks
- formatting
- lightweight user logic
- custom actions
- state transformations

Native functionality should remain Rust.

Example API concept:

```text
audio.volume
audio.muted

network.ssid
network.strength

workspace.active

player.title
player.artist
```

Rune should not be required for normal built-in modules.

---

# Configuration

Prefer declarative configuration for normal users.

Potential split:

```text
config.toml
    -> what exists / layout / modules

theme.scss
    -> visual styling

Rune
    -> behavior / custom logic
```

Do not make the complete configuration itself a Rune program.

---

# Styling

Long-term theme pipeline:

```text
CSS / SCSS / Sass
        ↓
optional Sass compiler
        ↓
restricted CSS parser
        ↓
StyleIR
        ↓
typed Rust Theme
        ↓
Slint theme properties
```

Slint should never need to interpret CSS directly.

The CSS-like frontend is only an authoring language.

Potential supported subset:

```text
color
background
border-color
border-width
border-radius

font-family
font-size
font-weight

padding
gap

width
height
min-width
max-width

opacity
transition-duration
```

Potential semantic selectors:

```css
.workspace {
}
.workspace.active {
}
.workspace.urgent {
}

.audio {
}
.audio.muted {
}

.battery {
}
.battery.critical {
}
```

Avoid implementing browser complexity such as:

```text
full CSS cascade
arbitrary descendant selectors
!important wars
browser layout semantics
float
z-index
full CSS grid
```

Layout remains Slint's responsibility.

---

# Surface Model

Create a generic surface abstraction.

Possible model:

```rust
WidgetPlacement {
    output: OutputPreference,
    anchor: Anchor,
    offset: Point,
    size: Size,

    pinned: bool,
    movable: bool,
    dismiss_on_blur: bool,
}
```

Potential surface types:

```text
Bar
Popup
Floating
Pinned
OSD
Notification
```

The UI component itself should remain independent from this.

---

# Movable Widgets

Desired UX:

```text
normal click
-> open popup

Ctrl+Alt+Drag
-> enter move mode

drag widget
-> reposition surface

release
-> persist placement
```

Possible later UX:

```text
popup
-> drag away
-> pin
-> becomes persistent desktop widget
```

Potential reverse behavior:

```text
floating widget
-> drag onto bar
-> dock
```

Do not implement docking in the first PoC unless trivial.

---

# Placement Persistence

Do NOT store only:

```text
x = 3142
y = 817
```

Store semantic placement:

```toml
[placement]
connector = "DP-2"
fingerprint = "vendor:model:serial"
anchor = "top-right"
offset = [24, 80]
```

Connector names alone are not reliable.

Where available, retain physical display identity:

- connector
- manufacturer
- model
- serial / EDID fingerprint

---

# Output Fallback Resolver

This is a first-class subsystem.

Target behavior:

```text
preferred output available?
    ↓ no

same physical monitor fingerprint?
    ↓ no

same logical role?
    ↓ no

spawn/source output?
    ↓ no

preferred/primary output?
    ↓ no

any active output?
    ↓ no

defer surface creation
```

Possible default resolution order:

```text
1. exact connector
2. same monitor fingerprint
3. matching logical role
4. output that spawned the widget
5. primary/preferred output
6. any active output
7. defer until an output exists
```

---

# Saved Placement vs Runtime Placement

These MUST remain separate.

Example:

```text
Saved preference:
DP-2 / top-right / +24,+80

Runtime fallback:
eDP-1 / top-right / +24,+80
```

If DP-2 disappears:

```text
widget moves to fallback
```

But DO NOT overwrite the saved preference.

When DP-2 returns:

```text
widget automatically returns home
```

Hard invariant:

> Temporary topology changes must not destroy user placement intent.

---

# Reachability

Another hard invariant:

> A widget must never become unreachable because display topology changed.

After placement resolution:

```text
resolve output
-> resolve anchor
-> apply offset
-> calculate final geometry
-> clamp to usable output area
-> verify minimum visible/grabbable region
```

If still invalid:

```text
center safely on resolved fallback output
```

If no output currently exists:

```text
defer
```

Do not spawn surfaces at arbitrary `(0,0)` when output state is invalid.

---

# Output Hotplug Behaviour

Output topology changes must be treated as runtime events.

Example:

```text
widget preference:
DP-2

DP-2 disappears
↓
resolve temporary fallback
↓
move/recreate surface on fallback output
↓
keep original preference intact

DP-2 returns
↓
resolver detects preferred target
↓
restore widget to original placement
```

This must work without restarting Owlenbar.

Monitor disconnect/reconnect should not corrupt persisted placement.

---

# Compositor Independence

Avoid coupling core services to Hyprland.

Expected possible compositor support later:

```text
Hyprland
Sway
niri
Mango
others
```

Use adapters where compositor-specific functionality is unavoidable.

Concept:

```text
CompositorBackend trait
├── HyprlandBackend
├── SwayBackend
├── NiriBackend
└── MangoBackend
```

Generic Wayland protocols should be preferred wherever possible.

A compositor-specific integration must not leak through the rest of the architecture.

Owlenbar should remain usable when the user changes compositor.

---

# Wayland Strategy

Use the right protocol/surface type for the job.

Expected rough mapping:

```text
bar
-> layer-shell

OSD
-> likely layer-shell

notifications
-> likely layer-shell or dedicated surfaces

popup
-> dedicated small popup/surface

floating widget
-> dedicated surface managed by shell runtime
```

Do not blindly implement everything as one giant layer-shell surface.

Investigate protocol constraints carefully, especially around positioning.

Wayland does not allow arbitrary absolute positioning of normal client windows in the same way X11 does.

For persistent shell widgets, semantic placement via:

```text
output + anchor + margins/offset
```

is preferred.

The exact protocol strategy must be validated before committing to the architecture.

---

# Atomic Surface Requirement

The first implementation must explicitly prove that:

```text
visible geometry == interactive surface geometry
```

For example, if the calendar visually occupies:

```text
420 × 600 px
```

there must not secretly be something equivalent to:

```text
3440 × 1440 transparent surface
└── 420 × 600 calendar
```

unless there is an unavoidable protocol limitation.

If such a limitation exists, document it rather than hiding it.

---

# First Vertical Slice

DO NOT attempt to build a complete bar yet.

The first PoC should prove the unique architecture.

Implement:

```text
Rust
+
Slint
+
Wayland output discovery
+
SurfaceManager
+
PlacementResolver
+
one Bar
+
one Clock widget
+
one Calendar popup
```

Required flow:

```text
launch
↓
bar appears
↓
clock visible
↓
click clock
↓
atomic calendar surface opens
↓
Ctrl+Alt+Drag calendar
↓
move calendar
↓
drop calendar
↓
placement saved
↓
restart owlenbar
↓
calendar restores at saved position
```

Then test topology behavior:

```text
calendar saved on DP-2
↓
disconnect DP-2
↓
calendar moves to fallback output
↓
placement remains reachable
↓
reconnect DP-2
↓
calendar automatically returns to saved location
```

THIS is the main architecture test.

---

# First PoC Non-Goals

Do not implement yet:

- PipeWire mixer
- NetworkManager
- Bluetooth
- notification daemon
- system tray
- MPRIS
- launcher
- lockscreen
- clipboard
- full Rune extension API
- complete SCSS system
- full compositor compatibility
- fancy plugin marketplace
- Waybar feature parity

Stub these if necessary.

Do not spend weeks writing system modules before proving the surface model.

---

# Performance Goals

Do not prematurely micro-optimize, but design for:

```text
first visible shell:
< 200 ms target

idle CPU:
effectively 0% when no state changes occur

polling:
avoid where event-driven sources exist

normal bar PSS:
target < 50 MiB if realistic
```

These are targets, not contractual requirements.

Measure rather than assume.

---

# Event Model

Prefer event-driven system state.

Avoid:

```text
poll every 200 ms
poll every second
spawn command repeatedly
```

when DBus, Wayland events, PipeWire callbacks, inotify, netlink, etc. can provide updates.

Lessons from the current AGS setup:

- avoid top-level synchronous polling during startup
- do not spawn subprocesses for trivial state
- render base shell first
- initialize non-critical state asynchronously
- stagger expensive background tasks if needed

---

# Startup Philosophy

Expected sequence:

```text
process start
↓
load config
↓
initialize output state
↓
create basic surfaces
↓
first paint
↓
core shell ready
↓
initialize non-critical services asynchronously
↓
widgets update as data arrives
```

Do NOT require all optional services to become ready before first paint.

---

# Repository Structure Proposal

Evaluate this rather than blindly following it:

```text
owlenbar/
├── crates/
│   ├── owlenbar-core/
│   ├── owlenbar-wayland/
│   ├── owlenbar-placement/
│   ├── owlenbar-ui/
│   ├── owlenbar-script/
│   └── owlenbar-services/
│
├── ui/
│   ├── components/
│   └── theme/
│
├── config/
│
└── src/
```

For the initial PoC, avoid unnecessary workspace fragmentation.

A single crate with clean modules may be preferable until boundaries become clear.

Do not architecture-astronaut the initial implementation.

---

# What I Want From You First

Before implementing:

1. Inspect the current Slint + Wayland ecosystem and APIs.

2. Identify the cleanest way to create:
   - layer-shell surfaces
   - independent popup/floating surfaces
   - output-aware surfaces

3. Determine whether Slint's existing window/backend APIs are sufficient directly or whether a custom Wayland/windowing integration is required.

4. Determine how resizing a native surface based on Slint content should work.

5. Identify constraints around moving independent surfaces under Wayland.

6. Investigate which protocols can provide the desired atomic and persistent placement semantics.

7. Determine how output identity can be represented robustly across:
   - connector rename
   - hotplug
   - docking/undocking
   - compositor restart

8. Determine how modifier + pointer drag can be implemented robustly for:
   - Ctrl+Alt+Drag
   - cross-output movement
   - surface recreation if required

9. Propose the smallest architecture capable of implementing the Vertical Slice.

10. Explicitly list any protocol/compositor limitations.

Do not assume the architecture described here is automatically possible exactly as written.

Validate it against actual Wayland and Slint behaviour first.

---

# Implementation Strategy

After the investigation:

1. Document findings.

2. Select the smallest viable Wayland/Slint architecture.

3. Implement output enumeration and hotplug tracking.

4. Implement `SurfaceManager`.

5. Implement `PlacementResolver`.

6. Implement persistent placement state.

7. Implement one bar surface.

8. Implement one clock component.

9. Implement one atomic calendar surface.

10. Implement Ctrl+Alt+Drag placement.

11. Implement output fallback.

12. Implement automatic preferred-output restoration.

13. Add instrumentation.

14. Test restart and hotplug behaviour.

Do not proceed into unrelated shell features until this works.

---

# Required Tests

At minimum test:

```text
cold launch
warm launch

single monitor
multiple monitors

preferred monitor present
preferred monitor absent

monitor disconnect while popup is open
monitor reconnect while popup is open

restart with preferred monitor present
restart with preferred monitor absent

connector identity changes where physical identity survives

saved offset larger than fallback output

saved widget geometry larger than fallback work area

all outputs temporarily disappear

output appears after startup

rapid disconnect/reconnect
```

Placement resolution should be unit-testable without requiring a real compositor.

For example:

```rust
resolve(preference, available_outputs) -> RuntimePlacement
```

should ideally be mostly pure logic.

---

# Required Instrumentation

Include startup milestones such as:

```text
[+   0ms] process start
[+  ??ms] config loaded
[+  ??ms] Wayland connected
[+  ??ms] outputs ready
[+  ??ms] Slint initialized
[+  ??ms] bar surface created
[+  ??ms] first frame
[+  ??ms] shell ready
```

Also log placement decisions at debug/trace level:

```text
widget calendar:
preferred DP-2 unavailable
fingerprint match unavailable
spawn output unavailable
using primary DP-1
placement clamped from ...
```

When DP-2 returns:

```text
widget calendar:
preferred output restored: DP-2
runtime placement migrated DP-1 -> DP-2
```

Do not make normal operation noisy at info level.

---

# Architectural Invariants

Treat these as hard requirements unless Wayland itself makes one impossible.

### Invariant 1

> No invisible interaction geometry.

### Invariant 2

> Widget content does not own placement.

### Invariant 3

> Temporary topology changes do not destroy persisted user intent.

### Invariant 4

> A persisted widget must never become unreachable.

### Invariant 5

> Compositor-specific behaviour stays behind adapters.

### Invariant 6

> Non-critical services do not block first paint.

### Invariant 7

> Prefer events over polling.

### Invariant 8

> Rune does not directly mutate arbitrary Slint internals.

### Invariant 9

> Runtime fallback placement is not persisted as the user's preferred placement.

### Invariant 10

> If no safe placement currently exists, defer rather than inventing one.

---

# Long-Term Direction

If the Vertical Slice succeeds, Owlenbar can gradually evolve from a bar into a native Wayland shell runtime.

Potential future scope:

```text
bar
workspaces
tray
MPRIS

audio / PipeWire
network
Bluetooth
power / battery
brightness

notifications
OSD
launcher
clipboard

desktop widgets
persistent widgets
widget docking

idle integration
lockscreen integration

Rune extension API
SCSS/Sass theme frontend
IPC / CLI

compositor adapters
```

But these are future layers.

The initial project should remain focused on proving the underlying surface architecture.

---

# Product / Architecture Thesis

The key idea is:

> Widgets aren't children of the shell. They're independently placeable UI units.

A widget's content, presentation, system state and physical placement are separate concerns.

Conceptually:

```text
                  Widget Content
                       │
                    Slint
                       │
                 typed state
                       │
                Widget Runtime
                       │
          ┌────────────┼────────────┐
          │            │            │
         Bar          Popup       Floating
          │            │            │
          │            │          Pinned
          │            │            │
          └────────────┼────────────┘
                       │
                 SurfaceManager
                       │
                PlacementResolver
                       │
                    Wayland
```

A future user interaction could therefore become:

```text
click bar widget
↓
popup opens
↓
Ctrl+Alt+Drag
↓
detach
↓
drag across outputs
↓
drop
↓
persist
↓
widget is now pinned/floating
```

Potentially later:

```text
drag floating widget onto bar
↓
dock
↓
independent surface disappears
↓
same content becomes bar child
```

This is deliberately more general than a traditional status bar.

---

# Success Criteria

The experiment succeeds if:

- the bar starts quickly
- the calendar is an actually atomic surface
- no invisible surrounding surface intercepts input
- Ctrl+Alt+Drag can reposition it
- placement survives restart
- output disconnect triggers a valid fallback
- the original placement preference is preserved
- reconnecting the preferred monitor restores the widget
- widgets cannot become permanently unreachable
- topology changes are handled live
- the UI remains responsive
- architecture does not depend on GTK or Qt
- compositor-specific code is isolated
- placement logic is independently testable

If the desired surface behaviour is impossible using generic Wayland protocols, stop and clearly document:

- exactly which protocol constraint blocks it
- which parts remain possible
- compositor-specific alternatives
- architectural alternatives
- trade-offs of each option

Do not hide architectural limitations behind hacks.

---

# Immediate Task

Start with investigation and technical validation.

Do **not** start implementing the complete shell.

The first question to answer is:

> Can Rust + Slint provide genuinely atomic, independently positioned Wayland shell surfaces with persistent topology-aware placement, live output fallback and restoration, without GTK/Qt?

Then build the smallest Vertical Slice that proves or disproves it.

If that works, Owlenbar has its foundation.
