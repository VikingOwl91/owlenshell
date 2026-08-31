# Owlenshell Configuration

## Goals

Configuration should be:

- declarative by default
- usable without scripting
- versionable
- diagnosable
- hot-reloadable where practical
- separated from styling and behavior

## Responsibility Split

```text
Config
  → what exists

Rune
  → custom behavior

Style
  → visual appearance

Rust runtime
  → validated truth and system behavior
```

## Proposed Structure

Example only:

```toml
version = 1

[shell]
profile = "default"

[[surfaces]]
id = "main-bar"
type = "bar"
placement = "top"
mode = "cover"
output = "preferred"

[[surfaces.widgets]]
type = "workspaces"

[[surfaces.widgets]]
type = "window-title"

[[surfaces.widgets]]
type = "clock"

[widgets.calendar]
mode = "float"

[widgets.calendar.placement]
anchor = "top-right"
offset = [24, 80]

[input]
layout-edit = "CTRL+ALT+E"

[input.move]
binding = "CTRL+ALT+BTN_LEFT"
conflict-policy = "fallback"
move-handle = "auto"

[theme]
path = "~/.config/owlenshell/theme.scss"
```

## Defaults

A missing config should still produce a useful shell.

The default shell is part of the product, not an example users must copy.

## Placement

Built-ins:

- top
- bottom
- left
- right
- custom

Custom placement should compile into validated runtime policy.

## Modes

Built-ins:

- cover
- float

Custom modes should compile into validated surface policy.

## Bindings

Bindings should be user-configurable.

Conflict introspection is best-effort through compositor adapters.

Unknown conflict state must not be treated as known-free.

Explicit user configuration should not be silently replaced unless the configured policy explicitly allows fallback selection.

## Profiles

Future profile support may represent:

- desktop
- laptop
- docked
- gaming
- presentation

Automatic topology matching may select profiles, but user intent must remain inspectable and overridable.

## Validation

Invalid config should produce actionable errors.

Example:

```text
widgets.calendar.placement.anchor:
unknown value "upperish"

expected one of:
top, bottom, left, right, top-left, top-right, bottom-left, bottom-right, custom
```

## Hot Reload

Reload should be transactional:

```text
parse
→ validate
→ build new desired state
→ apply
```

If parsing or validation fails, keep the current valid configuration.

## Evolution

Configuration must be versioned.

Breaking changes should have migration support when Owlenshell reaches released compatibility commitments.
