# Future Ideas

This document captures promising post-v1 ideas that fall naturally out of Owlenshell's typed provider, atomic-surface, policy and diagnostics architecture. These are ideas, not v1 commitments.

The common theme is simple: Linux already exposes an enormous amount of useful desktop state, but it is fragmented across Wayland protocols, portals, PipeWire, WirePlumber, systemd/logind, D-Bus, udev, NetworkManager, `/proc`, cgroups and compositor-specific APIs. Owlenshell can make that state visible, understandable and directly manipulable through shared typed models.

## Desktop Portal Visibility

Turn normally invisible `xdg-desktop-portal` plumbing into understandable shell UX.

Potential surfaces:

- active screen-sharing sessions
- active remote-desktop sessions
- portal-mediated permissions and requests where observable
- file/open-URI activity where useful and appropriate
- clear recording/sharing indicators
- direct access to relevant controls and termination actions

The goal is not to replace portals, but to expose useful portal state through coherent native shell UI.

## Inhibitor Inspector

Expose systemd/logind and related inhibitors in the power/session UI.

Examples:

- which application prevents suspend
- which application prevents idle
- which application delays shutdown
- the reason supplied by the inhibitor
- how long the inhibitor has been active

A user should not need several terminal commands to answer: **Why won't my computer sleep?**

## Desktop User-Service Manager

Provide a deliberately scoped native view of desktop-relevant systemd user units rather than attempting to become a full systemd administration frontend.

Potential actions:

- inspect state
- start/stop/restart
- enable/disable where appropriate
- inspect recent logs
- surface failed desktop services
- connect service state to Owlenshell diagnostics

This should integrate naturally with UWSM-aware session handling.

## Interactive PipeWire Routing

Treat the PipeWire graph as a desktop primitive rather than only exposing a volume slider.

Potential UX:

- route individual applications to outputs via direct manipulation
- inspect active streams
- move streams between devices
- understand capture/playback relationships
- show application, device and stream state through a native Slint graph or simpler routing UI

Example:

```text
Firefox  -> Headset
Discord  -> Headset
Spotify  -> Speakers
```

## Persistent Per-Application Audio Policy

Build on the PipeWire/WirePlumber provider layer to support understandable persistent routing policy.

Examples:

- Spotify always prefers speakers
- Discord always prefers headset
- games prefer a configured gaming output
- microphone policy can follow application/profile context

The shell should expose policy and current routing separately so users can understand both intent and actual state.

## Device Arrival UX

Use udev/device events as first-class shell events rather than polling.

Examples:

- USB storage: open, mount, unmount/eject
- controller: battery, mapping/profile actions
- phone: relevant connection actions
- removable devices: concise health/state information

Device-specific functionality should remain provider-backed and capability-aware.

## Network and VPN Visibility

Expose more of Linux networking than a simple connected/disconnected Wi-Fi icon.

Potential information:

- active physical interface
- VPN/WireGuard tunnel state
- routing path where it can be represented reliably
- kill-switch state where available
- per-connection details
- useful namespace/container networking state in advanced diagnostics

Avoid pretending that per-application routing is known when it cannot be determined reliably.

## Why Is My Computer Awake?

Create a user-facing diagnosis that correlates power-management state across providers.

Potential inputs:

- systemd/logind inhibitors
- media playback
- active jobs/services
- screen sharing/recording
- intentional Owlenshell policies
- wakeup/power-management information where available

The output should answer the human question rather than dump subsystem internals.

## Policy-Based Do Not Disturb

Evolve Do Not Disturb from a notification toggle into a typed desktop policy.

Possible triggers:

- fullscreen application
- game running
- screen sharing
- presentation mode
- streaming/recording
- user-defined desktop mode

Possible effects:

- suppress or defer notifications
- reduce non-critical OSDs
- adjust idle behavior
- protect private notification content
- postpone non-urgent interruptions

Policies must remain inspectable and user-controllable.

## Clipboard Provenance and Privacy

Where provenance can be determined reliably, enrich clipboard history with source context and privacy policy.

Potential features:

- source application indication
- per-application history rules
- automatic expiry
- sensitive-entry handling
- never-store rules for known sensitive sources

Wayland limitations and unreliable provenance must be represented honestly rather than guessed.

## Temporary Desktop Modes

Provide coordinated desktop modes that apply typed policy across multiple providers and shell surfaces.

Examples:

### Presentation Mode

- Do Not Disturb
- inhibit suspend
- privacy-safe notifications
- presentation-oriented output policy

### Gaming Mode

- Do Not Disturb
- gaming-specific widgets/metrics
- audio/output policy
- optional performance-related integrations

### Streaming Mode

- suppress sensitive notification content
- expose OBS/recording state where integrations exist
- microphone state
- recording/sharing indicators
- streaming-specific widget layout

These should be composable policy profiles rather than opaque scripts.

## Cross-Subsystem Application Identity

Investigate a shared model that correlates application identity across otherwise separate Linux subsystems.

Potential relationships:

```text
Application
├── Wayland toplevel/window
├── PID/process tree
├── cgroup/systemd scope
├── PipeWire streams
├── desktop entry/app-id
├── resource usage
└── network context where reliably available
```

If reliable enough, this could enable a single application action surface containing window, process, audio, resource and service controls.

The correlation layer must expose confidence/limitations rather than inventing identity matches.

## Resource Inspector: "What the fuck is using this?"

A human-oriented diagnostic surface that answers which application currently owns or uses a desktop resource.

Example:

```text
Microphone
└── Discord
    ├── PipeWire node #71
    ├── PID 18432
    ├── app-id: discord
    └── active for 01:42:17

Camera
└── not in use

Audio output
├── Firefox
├── Spotify
└── Discord

Sleep
└── inhibited by Firefox
    └── reason: Playing video
```

Candidate resources include:

- microphone
- camera
- audio devices
- screen sharing/capture
- sleep/idle inhibitors
- removable devices
- selected network/VPN state

This should reuse provider state rather than implementing a second diagnostics stack.

## Architectural Opportunity

These ideas become disproportionately cheap once Owlenshell already owns normalized typed providers such as:

- `PipeWireProvider`
- `PowerProvider`
- `ToplevelProvider`
- `NetworkProvider`
- `DeviceProvider`
- `SystemdProvider`
- portal/session providers

The strategic opportunity is therefore not a collection of unrelated widgets. It is a shared desktop-state model that allows many small native UIs and policies to be built on top of the same event-driven data.

A useful long-term principle:

> **Make Linux desktop state visible, understandable and directly manipulable instead of leaving it fragmented across CLI tools and subsystem-specific interfaces.**

## v-infinity: Family IT Support Without the Phone Debugging

This section is intentionally **v-infinity**: capture the idea, preserve the architecture implications, and absolutely do not let it become a prerequisite for turning AGS off.

The problem is familiar: a less technical Linux user sees only the GUI symptom, while the person helping them needs service state, configuration health, logs and environment details. Remote diagnosis then degenerates into reading terminal commands over the phone and receiving incomplete screenshots.

Potential direction:

### Progressive Diagnosis

Present the useful answer before subsystem detail:

```text
Printing unavailable
CUPS configuration appears damaged.

[ Try safe repair ]  [ Details ]
```

Advanced details can correlate provider state such as service status, device detection, configuration health and relevant recent logs. The normal user should not need to understand systemd or CUPS merely to act on a known safe fix.

Principle:

> **Show the answer first. Explain the system on demand.**

### Trusted Support Contact

Optionally allow a user to configure one or more trusted support contacts -- including, inevitably, the dedicated family IT person.

A support request should not merely send "it doesn't work". Owlenshell can generate a structured diagnostic bundle containing only relevant information, with an explicit preview before sharing.

Potential bundle contents:

- Owlenshell version
- distribution/session/compositor information
- relevant provider and service states
- narrowly scoped recent logs
- configuration health signals
- relevant hardware/device state
- attempted or suggested remediation

Sensitive and unrelated data such as passwords, clipboard history, arbitrary home-directory contents and unrelated logs must not be silently included. Users should be able to inspect and redact the exact payload before sharing.

A configured support contact must not turn Owlenshell into an automatic escalation pipeline. Self-diagnosis and safe local remediation should remain the first path.

### Support Request UX

A support-capable Owlenshell instance could recognize a structured Owlenshell support bundle and present it as an actionable diagnostic view rather than a blob of text:

```text
Support request from Cousin-PC

Printing
Configuration appears damaged
CUPS: active
Printer: detected

[ Reply ] [ Request more diagnostics ] [ Connect remotely... ]
```

### Consensual Remote Assistance

As a much later extension, a support request could transition into an explicitly consented remote-assistance session.

Owlenshell should own the understandable workflow, consent UX, capability grants, diagnostics and revocation semantics rather than unnecessarily inventing a new remote-desktop protocol. Existing Linux primitives such as desktop portals and PipeWire, plus an appropriate transport/backend, should be reused where possible.

Remote access must be capability-scoped and visible to the assisted user. Example initial grant:

```text
Chris wants to help remotely

Requested access:
✓ View screen
✓ Control mouse & keyboard
✗ Read files
✗ Clipboard
✗ Terminal
✗ Administrator access

[ Decline ] [ Allow ]
```

Additional capabilities must require an explicit new grant. A helper who later needs terminal access, file access or privileged action should request that capability with a human-readable reason rather than silently escalating the session.

Potential grant properties:

- allow once / session-scoped access
- automatic expiry
- explicit capability list
- prominent active-session indicator
- immediate local termination control
- no hidden persistence
- automatic revocation when the session ends
- optional local session record showing what capabilities were granted and for how long

Privileged/root operations remain separate from ordinary remote-control permission and should use the normal constrained privileged-action/Polkit model rather than inheriting remote-session authority.

### Why This Fits Owlenshell

This is not a reason for Owlenshell to become a remote administration suite. It is a possible long-term consequence of foundations already useful elsewhere:

- typed diagnostics/providers can produce the support context
- capability policy can express temporary remote permissions
- portal visibility can expose screen/control sessions
- system-service/configuration diagnostics can explain failures
- the native shell can provide clear consent and session state

The end-to-end idea is:

```text
Diagnose
  -> offer safe self-fix
  -> create inspectable/redactable support bundle
  -> send to trusted support contact
  -> optionally request consensual remote assistance
  -> grant only required capabilities
  -> automatically revoke everything at session end
```

If this ever ships, its success criterion is fewer telephone conversations beginning with: **"Open a terminal. No, the other black window."**

## Scope Discipline

None of the ideas in this document should block the v1 acceptance contract.

The ordering remains:

1. Build a daily-driver shell good enough to turn the existing shell off.
2. Harden the shared provider/configuration/diagnostics foundations.
3. Add high-value integrations when the required state already exists in those foundations.
4. Avoid building a complete system administration suite merely because Linux exposes the data.
