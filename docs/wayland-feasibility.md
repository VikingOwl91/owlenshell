# Wayland / Slint Feasibility Record

Validated 2026-08-31 against Slint 1.17.1, Smithay Client Toolkit 0.21.1,
wayland-client 0.31.14, wayland-protocols 0.32.9, and the current protocol XML.
This document records what the MVP relies on and what generic Wayland cannot do.
Nothing below is worked around by hacks; limitations are accepted and visible.

## Verdict

Rust + Slint + SCTK can provide genuinely atomic, independently placed Wayland
shell surfaces with persistent topology-aware placement, live output fallback,
and restoration, without GTK/Qt. This requires a custom Slint platform built on
`wlr-layer-shell`. Slint's default winit backend cannot do it.

## Surface roles

| Role | Candidate | Verdict |
|---|---|---|
| Bar / OSD / floating widget | `wlr-layer-shell` (v4/v5) | Chosen. Output-specific, exact-size, anchored, margin-addressable. |
| Detached calendar | `wlr-layer-shell` | Chosen. |
| Normal window | `xdg_toplevel` | Rejected. Compositor-positioned; no absolute or output placement. |
| Popup | `xdg_popup` via `xdg_positioner` | Rejected for detached widgets. Must intersect or be adjacent to parent; transient by design. |

## Layer-shell facts the runtime relies on

- Exact `set_size(w, h)` is valid; visible geometry equals interactive geometry.
- Anchor + margins provide semantic corner/edge placement; updating margins on
  commit moves a surface within its output.
- A layer surface is bound to a `wl_output` at creation (or null for compositor
  choice). An existing layer surface cannot be rebound to another output;
  migration destroys and recreates the role while the Slint state object lives on.
- State is double-buffered and applied on `wl_surface.commit`; the initial
  commit must carry no buffer, then the configure is acknowledged before the
  first buffer attach.
- Input regions can be set explicitly to the visible content.
- `keyboard_interactivity=none` gives no keyboard focus; `on_demand` (v4+) uses
  compositor-defined focus behavior; `exclusive` is for lock screens and is not
  used here.
- Layer surfaces receive pointer/touch input normally; during a button press the
  implicit grab continues delivering motion/release for that surface.

`wlr-layer-shell` is an unstable extension protocol, not core Wayland. Support
must be probed at runtime; absence is a hard error for this shell.

## Custom Slint integration

- Slint's Linux backend is winit; it exposes `xdg_toplevel` handles only and has
  no layer-shell or output-placement API. `Window::set_position()` is
  documented as unavailable on Wayland.
- The supported seam is a custom `slint::platform::Platform` whose
  `create_window_adapter()` returns an adapter per component. Each Slint
  component (bar, calendar) gets its own adapter, `wl_surface`, and layer role.
- Rendering uses `MinimalSoftwareWindow` (feature `renderer-software`) drawing
  through `SoftwareRenderer` into `wl_shm` ARGB8888 buffers. It is `!Send`:
  Slint state and the Wayland event loop share one thread.
- Redraws are requested by Slint (`request_redraw` on the adapter) and by the
  runtime; `draw_if_needed` renders into the buffer, then the buffer is
  attached and committed. Frame callbacks throttle redraws.
- Input is delivered by dispatching `WindowEvent::Pointer*` via
  `Window::try_dispatch_event`; positions are logical (surface-local, scaled).
- Slint layout constraints drive the layer-shell size request. The compositor
  configure is authoritative, and the software window is resized to the actual
  size. If the compositor forces a different size, Slint fills it so no
  invisible interaction geometry exists.
- Scale factor: dispatch `WindowEvent::ScaleFactorChanged` from the output's
  scale; buffer pixel size is logical size x scale.

## Output identity

Generic Wayland exposes per-output: name (e.g. `DP-2`, optional, no guarantee
it maps to DRM), make, model, physical size (mm), transform, scale, modes, and
via xdg-output (optional, unstable): logical position/size and description.

- No serial, no EDID fingerprint, no persistent monitor ID.
- `wl_output.description` is free-form, non-unique, and not parsed for identity.
- Therefore `OutputPreference` stores connector/name plus make, model, physical
  size, and an optional serial that only a compositor adapter can supply.
- Matching is confidence-ranked: exact name, then unique serial
  (adapter-provided), then unique make/model/physical-size fingerprint, spawn
  output, preferred output, any active output, defer. Logical roles are not
  modeled in the MVP. Ambiguous serial or fingerprint matches fall through
  rather than guess.
- Logical output positions may be absent or all `(0,0)`; ambiguous hit tests do
  not select an output. The Hyprland adapter enriches serials and preferred
  output identity; geometry still comes from generic Wayland/xdg-output.
- Output removal synchronously evicts the output from both the placement
  snapshot and Wayland-object map before surfaces re-resolve. Hosted surfaces
  are destroyed or recreated against the remaining set before a later input
  callback can perform placement; a removed output is never a valid target.

## Modifiers and drag

- Pointer events never carry modifier state. Keyboard modifier updates are
  delivered only to surfaces with keyboard focus; a `none`-focus layer surface
  never sees them.
- `Ctrl+Alt+Drag` therefore works only when the calendar has focus
  (`on_demand`) and the compositor delivers modifiers to it. This remains a
  compositor-specific feasibility gate, not a portable guarantee.
- The portable fallback is a visible drag handle inside the widget (plain
  pointer drag, no global modifier observation). Both initiators are implemented.
- Core Wayland reports motion in coordinates relative to the focused surface.
  Moving that surface during the grab would invalidate the original delta, and
  core Wayland exposes no global pointer coordinate. The portable MVP therefore
  keeps the surface fixed during the grab and applies its final reachable
  placement on release. Live drag preview requires a separately validated
  relative-pointer or compositor adapter.

## Live PoC acceptance

The complete placement lifecycle was exercised on the Hyprland test system:

- The calendar was placed on `HDMI-A-1`, moved across outputs, persisted, and
  restored after an Owlenbar restart.
- Physically disconnecting `HDMI-A-1` migrated the open calendar to a remaining
  output without changing its saved preferred placement.
- Reconnecting `HDMI-A-1` returned the calendar to that output without user
  intervention.
- The neon debug styling made the native surface bounds visible throughout the
  test. The calendar remained an exact-size atomic surface; no output-sized
  transparent interaction surface appeared.

The LG display's physical power button has different semantics from cable
removal. Powering the panel off briefly disconnects and reconnects its Wayland
output, after which the compositor continues reporting the output as connected
and active while the panel remains off. Generic Wayland and the tested
compositor state therefore cannot reliably reveal the panel's physical power
state. Owlenbar correctly follows the reported output topology; this behavior
is not a hotplug failure.

## Explicit limitations (accepted, not hacked around)

1. `wlr-layer-shell` is an unstable wlr extension; no support means no shell.
2. Generic Wayland exposes no monitor serial or EDID; physical identity is
   best-effort and exact only via compositor adapters.
3. Generic Wayland has no primary-output concept; "preferred" comes from the
   adapter or a configured role.
4. Generic Wayland does not expose a universal usable work area; reserved-area
   info comes from the adapter when available, else the full output bounds.
5. Logical output coordinates may be unavailable or degenerate; cross-output
   drag targeting is exact only with an adapter, otherwise limited to
   same-output movement.
6. An unfocused surface cannot observe global modifier state.
7. Cross-output movement requires destroy/recreate of the layer role.
8. Fractional scaling is not exercised by the first smoke tests (live
   monitors are scale 1); integer scale is supported.
9. Generic drag relocation is applied on release; live surface movement during
   the grab is deferred until a safe relative-motion source is validated.
