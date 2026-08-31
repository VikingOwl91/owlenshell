# Owlenshell Extensions

## Goal

Owlenshell is batteries-included but programmable.

Extensions should increase flexibility without moving critical shell behavior into an unsafe or fragile scripting path.

## Principle

> Built-ins are native. Extensions are optional.

Rune is the intended scripting layer.

## Allowed Extension Categories

Potential extension types:

- custom formatter
- custom action
- event hook
- state transformation
- lightweight custom widget
- custom provider
- custom placement policy
- custom surface mode

Not all categories must be exposed in the first Rune milestone.

## Typed API

Rune should receive typed, validated state.

Conceptual examples:

```text
audio.volume
audio.muted

network.connected
network.ssid
network.strength

media.title
media.artist
media.playing

battery.percent
battery.charging
```

Actions should also be typed.

```text
audio.set_volume(...)
media.play_pause()
widget.open(...)
widget.close(...)
```

## Prohibited Direct Access

Rune must not directly receive:

- raw Wayland object handles
- arbitrary mutable Slint widget references
- PAM secrets
- unrestricted pointers/FFI
- privileged authentication internals

## UI Boundary

Avoid:

```text
find_widget("foo").set("background", ...)
```

Prefer:

```text
Rune
→ state/action
→ validation
→ Rust
→ Slint
```

## Failure Isolation

A failing script should be disabled or degraded locally.

The shell should continue.

Script errors should be observable through logs and diagnostics.

## Resource Limits

The runtime should eventually support limits such as:

- execution budget
- memory budget
- event-rate protection
- recursion protection
- cancellation

A script must not be able to stall the desktop indefinitely.

## Security Domains

Ordinary extension domain:

- widgets
- formatting
- automation
- actions

Protected domain:

- lock
- PAM
- authentication
- privileged session operations

No arbitrary extension code crosses into protected auth logic.

## Native Extensions

If a future native plugin ABI is added, it should be treated separately from Rune because native plugins carry a much larger trust and crash surface.

Do not create a stable native ABI prematurely.
