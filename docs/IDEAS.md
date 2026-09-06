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

A support contact should be a cryptographic identity, not merely an email address or account name. Email, Matrix or another channel may transport a request, but transport identity must not be the security boundary.

A possible model is a small modern public-key identity built around Ed25519/X25519-style primitives, or another well-reviewed equivalent. Existing OpenPGP or SSH identities may be useful as import/verification mechanisms, but the protocol should not inherit their complexity merely for familiarity.

Conceptually:

```text
Supporter identity
├── human-readable name: Chris
├── public signing key
├── encryption key/material
├── fingerprint
├── enrollment timestamp
└── verification method
```

Enrollment should be a deliberate operation. In-person QR-code exchange, fingerprint comparison or another out-of-band verification path is preferable for high-trust support relationships.

**Do not use first-contact TOFU as sufficient authorization for remote control.** A scammer who is already talking a user through a support flow must not be able to turn "accept this new key" into the remote-access installation step.

The relationship should ideally be bidirectional: the assisted device can verify the supporter, and the supporter can verify that a request came from an enrolled device rather than merely trusting the sender address.

A support request should not merely send "it doesn't work". Owlenshell can generate a structured diagnostic bundle containing only relevant information, with an explicit preview before sharing. Bundles can be signed by the device and, where appropriate, encrypted to the intended support contact.

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

A support-capable Owlenshell instance could recognize a signed structured Owlenshell support bundle and present it as an actionable diagnostic view rather than a blob of text:

```text
Support request from Cousin-PC
Identity: verified enrolled device

Printing
Configuration appears damaged
CUPS: active
Printer: detected

[ Reply ] [ Request more diagnostics ] [ Connect remotely... ]
```

The assisted side should likewise distinguish an enrolled, cryptographically verified supporter from a new or unknown identity. A display name alone is never evidence of identity.

### Consensual Remote Assistance

As a much later extension, a support request could transition into an explicitly consented remote-assistance session.

Owlenshell should own the understandable workflow, consent UX, capability grants, diagnostics and revocation semantics rather than unnecessarily inventing a new remote-desktop protocol. Existing Linux primitives such as desktop portals and PipeWire, plus an appropriate transport/backend, should be reused where possible.

Remote access must be capability-scoped and visible to the assisted user. Prefer the least powerful useful support mode. Screen viewing plus pointer/annotation may solve many problems without granting input control at all.

Example initial grant:

```text
Chris wants to help remotely

Identity: VERIFIED
Trusted supporter since: 2027-03-14

Requested access:
✓ View screen
✓ Point/annotate
✗ Control mouse & keyboard
✗ Read files
✗ Clipboard
✗ Terminal
✗ Administrator access

[ Decline ] [ Allow once ]
```

Additional capabilities must require an explicit new grant. A helper who later needs input control, terminal access, file access or privileged action should request that capability with a human-readable reason rather than silently escalating the session.

Potential grant properties:

- allow once / session-scoped access
- automatic expiry
- explicit capability list
- prominent active-session indicator
- immediate local termination control
- no hidden persistence
- automatic revocation when the session ends
- optional local session record showing what capabilities were granted and for how long
- clear separation between viewing, annotation, input control, clipboard, files, terminal and privileged operations

Privileged/root operations remain separate from ordinary remote-control permission and should use the normal constrained privileged-action/Polkit model rather than inheriting remote-session authority.

### Scam and Social-Engineering Threat Model

Cryptography can answer **"Is this request signed by the enrolled key?"** It cannot answer **"Is somebody manipulating the user into enrolling the attacker?"** Both problems must be treated independently.

Remote support is therefore one of the rare Owlenshell features where deliberate friction may be a security property rather than a UX defect.

Potential safeguards to investigate:

- never allow a newly introduced identity to silently gain the same trust presentation as a long-established supporter
- prevent or heavily constrain supporter enrollment from inside an unsolicited incoming remote-support flow
- clearly distinguish `new/unknown`, `enrolled`, and `verified` identities
- display how and when a supporter was enrolled
- require separate consent for meaningful capability escalation
- avoid repeated low-information prompts that train users to press Allow
- include contextual anti-scam language for dangerous grants rather than generic permission text
- never claim that Owlenshell, a bank, Microsoft, package updates, refunds or malware removal inherently require remote support
- provide a single obvious local kill switch for every active session
- ensure support sessions cannot create hidden persistence or silently enroll new supporters

The design goal is stronger than preventing technically unauthorized access:

> **A legitimate user should have a realistic chance of recognizing when they are being manipulated into authorizing malicious remote access.**

This needs practical human threat modeling, not only protocol review.

### External Review Gate

Remote control must not graduate from v-infinity merely because the implementation works.

By the time this is considered seriously, Owlenshell should already be a mature enough project to provide reviewers with an actual implementation, protocol description, threat model and reproducible test environment rather than a speculative concept.

Before enabling general remote-control functionality, seek independent review from several perspectives:

- offensive security / security engineering
- fraud, scam and social-engineering practitioners/researchers
- Linux desktop/Wayland/portal security expertise
- accessibility and human-factors/UX expertise
- independent protocol/cryptographic review where custom protocol elements exist

Potential outreach examples, if appropriate at that future point, include practitioners/educators such as John Hammond, Scammer Payback/Pierogi, NetworkChuck, and others with directly relevant experience. These names are examples of useful perspectives, not endorsements, commitments or dependencies.

Useful review questions include:

- **Security:** How would you compromise or abuse this implementation?
- **Fraud/social engineering:** How would you talk a victim into defeating these safeguards themselves?
- **Human factors:** Can a normal user understand what is being granted without learning the underlying cryptography?
- **Linux integration:** Are portal/session/capability boundaries represented accurately?

Any serious remote-control release should treat unresolved high-impact findings from this review as blockers.

### Dependency Ladder

The remote-support idea should emerge from foundations Owlenshell already needs rather than pulling those foundations forward solely for remote support:

```text
v1
├── daily-driver shell
├── typed providers
├── settings and diagnostics
├── i18n
└── AGS can be turned off

later
├── capability model
├── sandbox/policy foundations
├── deeper Linux visibility
├── portal integration
└── mature diagnostics

much later
├── structured support bundles
├── cryptographic support identities
├── trusted-contact enrollment
└── passive/read-only assistance

v-infinity
└── remote control
    ├── social-engineering threat model
    ├── external security review
    ├── accessibility/human-factors review
    └── only then consider general availability
```

The existence of this roadmap must not pull remote-support work into v1 or delay core shell functionality.

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
  -> sign/encrypt for an enrolled support identity
  -> send over an untrusted transport
  -> optionally request consensual remote assistance
  -> verify both identities
  -> start with the least powerful useful capability set
  -> explicitly grant any escalation
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
