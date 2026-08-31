# Atomic Surfaces

## Purpose

Atomic surfaces are the defining runtime primitive of Owlenshell.

This document defines the model and invariants independently from any particular widget.

## Definitions

### Widget

A logical UI component with state and actions.

Examples:

- clock
- calendar
- audio control
- notification card
- workspace list

A widget is not necessarily a Wayland surface.

### Host

A runtime container in which a widget is rendered.

Examples:

- bar
- popup
- floating surface
- pinned surface
- notification stack

### Atomic Surface

A native Wayland shell surface whose interaction geometry corresponds to its intended visible UI region.

An atomic surface owns:

- native surface lifetime
- geometry
- input region
- output placement
- mode
- placement policy
- focus policy
- migration behavior

## Core Invariants

### 1. No invisible interaction geometry

A small visible widget must not require a full-output transparent input surface.

### 2. Content does not own placement

Widget content is independent from output, anchor and runtime fallback.

### 3. Saved placement is not runtime placement

Temporary fallback never mutates persisted user intent.

### 4. Hidden means non-interactive

A hidden atomic surface must not continue intercepting pointer or keyboard input.

### 5. Widget state survives surface recreation

Cross-output migration may require destroying and recreating a native Wayland role.

The logical widget must survive.

### 6. Removed outputs cannot win placement

Once an output is unavailable for placement, no stale snapshot may resolve a visible widget back onto it.

### 7. Reachability is guaranteed

Topology changes must not make a user-controlled widget permanently inaccessible.

### 8. Ambiguity is not identity

If two outputs cannot be distinguished safely, Owlenshell must not pretend they are uniquely identified.

### 9. Drop persists user intent

Only explicit user placement changes should update saved placement.

Automatic fallback/recovery must not.

### 10. Atomic does not mean one surface per visual primitive

A bar can be one atomic surface containing many child widgets.

## Hosting Examples

```text
BarSurface
├── WorkspaceWidget
├── TitleWidget
├── ClockWidget
└── TrayWidget
```

```text
CalendarWidget
      ↓
FloatingAtomicSurface
```

```text
NotificationStackSurface
├── NotificationWidget
├── NotificationWidget
└── NotificationWidget
```

The exact hosting strategy may vary where it preserves atomic interaction geometry.

## Placement

Persisted placement should contain:

```text
OutputPreference
Anchor
Offset
Mode
Optional size/constraints
```

Avoid permanent global desktop coordinates.

## Output Identity

Possible identity inputs:

- connector
- make
- model
- physical size
- serial
- compositor-enriched identifiers
- logical role

Never persist transient Wayland global IDs.

## Fallback

Default resolution order:

1. exact connector
2. strong physical identity
3. unique fingerprint
4. logical role
5. spawn output
6. preferred/focused output
7. deterministic active output
8. defer

## Return-Home

If runtime fallback differs from saved placement and the preferred output becomes valid again:

```text
fallback output
      ↓
preferred output returns
      ↓
re-resolve
      ↓
migrate surface
      ↓
saved placement remains unchanged
```

## Live Movement

The production system must visually update movement during drag.

Same-output movement should update native placement continuously where possible.

Cross-output movement may require:

- temporary preview
- adapter-assisted global coordinates
- surface recreation
- state-preserving migration

The chosen implementation must not corrupt pointer deltas by treating surface-relative motion as desktop-global motion.

## Input

Movement must not steal ordinary widget interaction.

Supported mechanisms should include:

- configurable modifier + drag
- visible move handle
- layout/edit mode

The portable fallback must always remain available.

## Modes

### Cover

May reserve desktop space.

### Float

Does not reserve desktop space.

Custom modes may extend these policies but must remain validated.

## Lifecycle

Typical floating widget lifecycle:

```text
create logical widget
→ resolve placement
→ create native surface
→ attach Slint host
→ render
→ interact
→ migrate/recreate if required
→ hide/destroy native surface
→ logical state may remain
```

## Testing

Placement logic should be unit-testable without a real compositor.

Live acceptance must cover:

- same-output move
- cross-output move
- persistence
- restart
- physical disconnect
- fallback
- return-home
- rapid hotplug
- scale changes
- oversized geometry
- no-output state
- stale callbacks after recreation
- input outside visible geometry
