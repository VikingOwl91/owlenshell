# Native Sharing and Shared Sessions

> **Status:** post-v1 exploration. This must not block the daily-driver shell or the milestone where AGS can be turned off.

Owlenshell's eventual PipeWire, portal, toplevel, application-identity, policy and session foundations create an opportunity to make Wayland-native screen sharing substantially more coherent than a generic "share screen" dialog.

The goal is not to reimplement OBS, Discord, Steam Remote Play or every transport stack. Owlenshell should own the desktop-native UX, source selection, privacy policy and session model, while reusing suitable Linux primitives and proven transport/codec implementations underneath.

## Why Owlenshell Is Well Positioned

A shell can understand more context than a standalone capture application:

- which output, region or Wayland toplevel the user selected
- which application/process identity is associated with that toplevel where correlation is reliable
- which PipeWire playback/capture nodes belong to the relevant application
- which microphone and output devices are active
- which notifications and shell surfaces are private
- whether Gaming, Streaming, Presentation or another desktop policy is active
- which surfaces should remain local even while the user switches workspaces or applications

This allows sharing to become a typed desktop operation rather than merely a stream of pixels.

A conceptual pipeline is:

```text
Wayland output / toplevel / region
              │
              ▼
      portal / capture source
              │
              ▼
        PipeWire video ─────┐
                            ├──> encode / transport ──> receiver
selected PipeWire audio ────┤
                            │
microphone (optional) ──────┘
```

The exact capture and transport mechanisms should remain replaceable implementation details.

## Audio Is a First-Class Share Source

Do not reduce audio sharing to a single `Share system audio` checkbox.

Where PipeWire application identity can be correlated reliably, allow users to select audio streams independently:

```text
Share with Matze

Video
● Window: Game
○ Display: DP-2
○ Region

Audio
☑ Game
☐ Discord
☐ Spotify
☑ Microphone

Privacy
☑ Hide notifications
☑ Keep non-shared windows private

[ Start sharing ]
```

A particularly valuable target is application-scoped sharing: selecting a game or browser window can offer its associated audio without automatically leaking every other desktop audio stream.

Cross-subsystem identity must remain honest about confidence. Owlenshell must not silently attach unrelated audio because a PID/app-id correlation is uncertain.

## Shared Session Model

Treat screen sharing as one use of a more general `SharedSession` abstraction.

Potential session inputs/capabilities:

```text
SharedSession
├── video sources
│   ├── output
│   ├── toplevel/window
│   └── region
├── audio sources
│   ├── application streams
│   ├── system/mix
│   └── microphone
├── participants / identities
├── transport
├── privacy policy
├── optional voice/chat
├── optional pointer/annotation
└── optional narrowly scoped input capabilities
```

Different products can then be policy profiles over the same primitives rather than unrelated implementations.

## Pair Coding

A pair-coding session could deliberately share a small working set rather than the complete desktop:

```text
Shared Session with Matze
Mode: Pair coding

Shared
✓ Editor
✓ Terminal
✓ Browser :3000
✓ Application audio
✓ Microphone

Private
🔒 Notifications
🔒 Other workspaces
🔒 Password manager

Participant
👁 View shared windows
🖱 Pointer / annotation
⌨ Input: explicitly granted targets only
```

If safe and technically feasible, future input sharing should be scoped as narrowly as possible instead of treating remote keyboard/mouse control as authority over the entire desktop.

## Watch Parties

For watching films, videos or other media together, a session may combine video, application audio and voice.

Where an application exposes useful MPRIS state, Owlenshell can investigate synchronized play/pause/seek semantics in addition to ordinary audiovisual streaming. This is an optimization/integration opportunity, not a requirement: generic sources must still work through normal capture.

DRM, protected-content restrictions and application/platform terms must be respected rather than bypassed.

## Game Sharing

A game session can begin as low-risk spectating:

```text
Game Share

Game video       ✓
Game audio       ✓
Voice            ✓
Desktop          🔒 not shared
Remote input     ✗
```

A later extension could investigate explicitly granted remote-controller input, including local/remote player assignment for games that support it. Remote game input should be a separate capability from desktop remote control.

The important privacy property is that sharing a game should not imply sharing whatever appears after Alt-Tab. If the session is bound to the game toplevel/source, unrelated applications should remain private by default.

## Streaming Integration

Owlenshell should initially complement mature streaming software rather than replace it.

A Streaming Mode could provide a native control plane over desktop state and integrations such as OBS:

```text
Streaming Mode

OBS                    ● LIVE  01:42:17
Microphone             ● Shure MV7
Game audio             ● Stream
Discord                 ● Stream
Spotify                 ○ Local only
Notifications           🔒 Hidden
Screen capture          ● Game

[ Open OBS ] [ End stream ]
```

Potential long-term integrations:

- OBS status/control through supported APIs
- stream-aware PipeWire routing
- privacy-safe notification policy
- recording/sharing indicators
- capture-aware widget/surface policy
- selected Owlenshell widgets exposed as intentional stream overlays where technically appropriate

Do not build a second OBS merely because Owlenshell can see the same underlying state.

## Privacy by Construction

Native sharing should make privacy an explicit session policy rather than an afterthought.

Potential defaults and controls:

- application/toplevel sharing does not automatically become whole-desktop sharing
- notifications can be hidden or sanitized while sharing
- clipboard overlays/history remain private unless explicitly shared
- password-manager and other sensitive surfaces can remain excluded where reliable mechanisms exist
- unrelated workspaces/windows remain private
- active sharing is always visibly indicated
- the user can immediately terminate a session locally

Owlenshell must represent platform/compositor limitations honestly. It must not promise capture exclusion where the underlying stack cannot enforce it.

## Relationship to Remote Support

Shared sessions and remote support can reuse capture, audio, identity, transport, capability and privacy primitives, but they have different human threat models.

**Sharing:** the user intentionally wants to collaborate, watch or play with another participant.

**Support:** the user has a problem and another person is requesting access to help resolve it.

Remote support must therefore not become `SharedSession { control: true }` with a different label. Its stronger enrollment, anti-scam, capability-escalation and external-review requirements remain separate.

## Architectural Direction

Prefer reusable boundaries such as:

```text
ToplevelProvider ──┐
Portal/Capture ────┤
PipeWireProvider ──┼──> SharedSession model ──> session UX/policy
AppIdentity ───────┤                 │
PrivacyPolicy ─────┘                 ▼
                              transport backend
```

Possible transports/codecs should be evaluated later based on latency, interoperability, security and maintenance cost. The architecture should not prematurely couple the session model to one transport.

The key product opportunity is not "Owlenshell has screenshare." It is:

> **Owlenshell can combine Wayland capture, PipeWire audio, application identity and desktop privacy policy into one understandable sharing operation.**

## Scope Discipline

This is a post-v1 direction. Build the provider, portal, application-identity and policy foundations because they independently improve the shell. Shared Sessions should become attractive because those foundations already exist -- not become an excuse to delay them or the core desktop.
