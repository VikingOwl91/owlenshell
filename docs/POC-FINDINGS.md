# Atomic Wayland PoC Findings

## Status

The initial Owlenshell PoC demonstrated that the core atomic-surface concept is feasible.

The PoC is evidence, not the production architecture.

## Proven Stack

The PoC successfully used:

```text
Slint custom Platform / WindowAdapter
+
Smithay Client Toolkit
+
wlr-layer-shell
+
wl_shm software rendering
```

without GTK or Qt.

## Proven Behaviors

Live testing demonstrated:

- a native layer-shell bar
- an independent calendar-sized atomic surface
- no output-sized transparent input surface around the calendar
- same-output placement
- cross-output placement with native-surface recreation
- widget state preservation across recreation
- semantic persisted placement
- restart restoration
- physical output disconnect fallback
- automatic return-home when the preferred output returns
- near-zero idle CPU in the minimal PoC
- very small baseline resident memory in the minimal PoC
- fast first paint in the minimal PoC

## Important Architectural Validation

The PoC validated:

### Custom Slint platform

Slint can be used as the presentation/rendering layer while Owlenshell owns Wayland surfaces directly.

### Atomic geometry

Small shell UI does not need to be represented by a full-output transparent surface.

### Placement separation

Saved placement and runtime placement can remain independent.

### Output migration

A native layer surface can be recreated on another output while preserving logical widget state.

### Fallback and return-home

A widget can temporarily migrate to a fallback output without mutating its persisted preference, then return automatically.

## Current PoC Limitations

The PoC intentionally does not represent final production behavior.

Known limitations include:

- movement may update only on drop rather than continuously
- minimal/no production widget design
- limited input policy
- limited multi-seat support
- incomplete fractional scaling acceptance
- minimal system services
- no Rune runtime
- no StyleIR/theme system
- no production config system
- no production notifications/OSD/clipboard/lock stack

## Monitor Power-Button Observation

One tested LG HDMI display behaves unusually when powered off:

1. Wayland/compositor reports a disconnect.
2. The output is immediately re-announced.
3. The panel remains physically off.
4. The compositor again reports the output as logically connected/active.

This means physical panel power state cannot be inferred reliably from Wayland/compositor state for that hardware.

A real cable/output removal correctly exercised Owlenshell fallback behavior.

## Runtime Identity Observation

A Wayland output global ID changes across some reconnect/reannounce cycles.

Therefore:

> Transient Wayland global IDs must never be persisted as physical monitor identity.

## Production Implication

The production architecture should preserve the PoC's successful primitive:

> **Native atomic Wayland shell surfaces controlled by Owlenshell, with Slint used as the UI renderer.**

The production codebase should not preserve PoC shortcuts merely because they already work.
