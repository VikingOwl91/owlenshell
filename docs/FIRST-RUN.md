# First-Run, Onboarding and Migration

## Purpose

Owlenshell should not greet a new user with an empty desktop, a missing configuration error, or an implicit requirement to understand the implementation stack before the shell becomes useful.

The first-run experience should make a complete desktop approachable to a Linux newcomer while preserving a direct path for experienced users who want to own the configuration themselves.

The guiding principle is:

> Start with a useful desktop, then make every layer inspectable and customizable.

The first-run UI must consume the same typed configuration and runtime APIs as the normal Settings application. It must not create a parallel wizard-only configuration model or hidden state that cannot later be edited normally.

## Entry Paths

The first launch should offer three first-class paths:

1. **Guided setup** -- configure a useful Owlenshell desktop through a small visual workflow.
2. **I'll configure it myself** -- immediately expose the configuration through the user's preferred tools without abandoning them at a filesystem path.
3. **Import existing setup** -- detect an existing shell/bar environment, analyze it read-only, preview a migration and import the parts that can be represented safely.

All paths converge on the same typed Owlenshell configuration model.

## Environment Detection

Before asking questions Owlenshell should inspect the environment and surface useful detected capabilities, including where practical:

- Wayland compositor and available compositor adapter
- outputs/displays and topology
- fractional scaling
- UWSM/session environment
- PipeWire
- network backend
- Bluetooth
- brightness/power capabilities
- terminal emulators
- graphical text/code editors
- terminal editors
- graphical file managers
- terminal file managers
- existing shell/bar configurations
- supported greeter/login-screen integrations

Detection should inform defaults rather than silently mutate the system.

## Guided Setup

The guided path should remain short and visual. It should cover decisions that materially affect the initial desktop, such as:

- bar position: top, bottom, left or right
- theme/appearance with live preview
- output/display placement where relevant
- sensible built-in widget selection
- primary launcher and important shell bindings
- optional startup/session integration

The default choices should produce a complete usable desktop without requiring community widgets or scripting.

An advanced user must be able to skip the guided path immediately.

## Configure It Myself

Choosing manual configuration must still provide a polished workflow.

Owlenshell should detect suitable installed applications and offer them directly, for example:

- Neovim/Vim/Helix and other terminal editors
- Zed, VS Code and other graphical editors
- Yazi and other terminal file managers
- Dolphin, Nautilus, Thunar and other graphical file managers
- a custom command/application

Terminal applications should automatically launch inside the user's resolved/preferred terminal emulator and open the relevant Owlenshell configuration location. Graphical editors/file managers should receive the appropriate project/configuration path directly.

The selected preference should be reusable by normal Settings and CLI actions rather than existing only during onboarding.

Expected later affordances include:

- `Open config`
- `Open docs`
- `Validate`
- `Reload`
- `owlenshellctl config edit`

These should share a common preferred-application/launcher resolution layer.

## Existing Shell and Bar Migration

Owlenshell should detect common existing shell/bar configurations where practical, including candidates such as:

- AGS
- QuickShell
- Waybar
- eww
- compositor configuration that provides relevant layout/binding/session context

The migration model is **analyze and translate**, not a promise to execute or perfectly convert arbitrary source code.

Importers should produce a common typed intermediate representation that can map recognizable concepts into Owlenshell configuration, for example:

- bar position and dimensions
- module/widget ordering
- common built-in widget equivalents
- output assignments
- theme tokens that can be mapped safely
- launcher/clipboard bindings
- other well-understood shell actions

Custom JavaScript, QML, scripts or unsupported widgets should be identified explicitly rather than silently discarded. Where useful, Owlenshell may preserve placement metadata, create a visible placeholder, link to migration guidance, or allow the user to ignore the unsupported component.

Potential importer architecture:

```text
AGS ---------\
QuickShell ---+--> importer --> typed migration model --> preview --> Owlenshell config
Waybar -------+
eww ----------/
```

Importer implementations should remain isolated from the core configuration model so support for another source environment does not require special cases throughout Owlenshell.

### Migration Safety

Migration must be conservative:

- source configuration is read-only
- original shell/bar files are never modified as part of analysis/import
- detected state is presented before application
- the user can preview what is recognized, translated, skipped or uncertain
- only sufficiently understood mappings are imported automatically
- generated Owlenshell state can be reviewed before becoming the active configuration
- temporary runtime observations must not overwrite persisted user intent

The user should always retain an obvious path back to the previous shell while evaluating Owlenshell.

## Optional System Integration

Privileged system integration must remain separate from normal shell execution. Owlenshell itself must not run as root merely to configure system-owned components.

Where supported, onboarding may offer optional integration such as updating a compatible greeter/login screen to match the selected Owlenshell appearance. Such actions should:

- clearly explain what will change and why privileges are required
- be optional/skippable
- use a narrowly scoped privileged helper or Polkit action
- avoid granting the normal Owlenshell process elevated privileges
- remain available later under Settings rather than being a one-time wizard-only operation

Greeter/display-manager ownership remains outside the core v1 shell requirement; supported integration can evolve independently.

## Completion

The final onboarding state should summarize the resulting desktop and make the next actions obvious. A successful first run should leave the user with working shell essentials rather than merely a generated configuration file.

The experience should work for both ends of the audience:

- a newcomer can install Owlenshell, choose a few understandable options and receive a coherent Wayland desktop
- an experienced user can reach their editor/configuration immediately, or migrate a recognizable existing setup, without fighting the onboarding UI

Long-term product goal:

> **Bring your desktop with you.**

Owlenshell should lower the cost of trying a modern Wayland shell without lowering the ceiling for users who want complete control.
