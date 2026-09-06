# Update Intelligence

> **Status:** Post-v1 idea. This must not block the v1 goal of replacing the current daily-driver shell.

Linux desktop users already run update watchers in panels and bars. Those watchers know that updates exist, and often know the installed and candidate versions, but usually reduce that useful state to a number such as `37 updates`.

Owlenshell can turn the same event into an optional, human-oriented **update impact preflight**: not "summarize every changelog", but **identify which updates are likely to affect this actual machine and explain only those**.

## Problem

A rolling-release user may have hundreds of packages installed. It is not realistic to expect a human to read hundreds of changelogs, release notes, project blogs and migration guides before every system update.

"It was documented in the changelog" is useful project hygiene, but it is not sufficient migration UX at system scale.

The computer already knows important parts of the join operation:

- which package is installed
- the installed version
- the candidate version
- which local configuration/features are actually in use
- which services, providers and desktop integrations are active

The useful question is therefore not:

> What changed in all of these packages?

It is:

> **Which of these changes are relevant to my actual system, and what do I need to do about them?**

## Provider Model

The existing update watcher can expose structured state rather than only a count:

```text
PackageUpdate {
    package: "hyprland",
    from: "0.56.0",
    to: "0.57.0",
    source: "arch",
    upstream: ...,
    release_notes: ...,
}
```

Conceptually:

```text
Package manager / AUR
        |
        v
   UpdateProvider
        |
        +---- Bar: update count
        +---- Settings: update details
        +---- UpdateIntelligence
                    |
          +---------+----------+
          |                    |
          v                    v
   upstream metadata      local context
   release notes          config/features
   changelogs             installed versions
   migrations             active integrations
   advisories             provider state
          |                    |
          +---------+----------+
                    v
               Impact Report
```

The update count remains cheap and useful even when no intelligence backend is enabled.

## Filter Before Using an LLM

An LLM should not receive hundreds of changelogs merely because hundreds of packages are installed.

Prefer deterministic and cheap filtering first:

```text
437 installed packages
        |
63 available updates
        |
14 releases with relevant upstream metadata
        |
5 contain breaking/deprecation/migration signals
        |
2 intersect with local configuration/features
        |
LLM analyzes those 2
```

Exact filtering mechanisms are an implementation detail and may evolve. Useful signals can include explicit breaking-change metadata, migration documents, deprecation markers, changed configuration keys, package metadata and known local usage.

## LLM/Agent Interface

The update provider should expose a **structured, read-only interface** that an LLM or local/remote agent can consume.

The model should receive bounded context such as:

- package identity
- source and upstream identity
- installed and target versions
- relevant release/migration material
- explicitly selected local configuration fragments
- detected usage of affected APIs/options/features
- diagnostics needed to establish impact

The output should itself be structured enough for Owlenshell to render and validate, for example:

```json
{
  "risk": "breaking",
  "affected": true,
  "reason": "The installed configuration uses an option removed by the target release.",
  "migration_available": true,
  "confidence": "high"
}
```

LLM conclusions are advisory unless independently verified. The UI must distinguish known machine-readable facts from model inference.

## UX

A normal update popup should emphasize actionable exceptions rather than produce an AI-generated wall of text:

```text
63 updates

✓ 61 appear routine

⚠ 2 may affect your desktop

Hyprland 0.56 -> 0.57
Your config uses 3 settings affected by this release.
Migration appears mechanical.
[ Explain ] [ Prepare migration ]

hyprlogin 1.2 -> 1.3
Your current session configuration may be incompatible.
Manual decision required.
[ Explain ]

[ Show all updates ]
```

The exact packages and versions above are illustrative, not compatibility claims.

The progressive-disclosure rule used elsewhere in Owlenshell applies here too:

> **Show the answer first. Explain the system on demand.**

## Migration Safety Boundary

Update analysis is **read-only by default**.

An LLM or analyzer discovering a migration must not silently rewrite configuration as a side effect of checking for updates or running the package manager.

A separate explicit migration workflow may:

1. identify the affected configuration
2. prepare a minimal change
3. produce a diff
4. validate the resulting configuration where tooling exists
5. explain anything that cannot be migrated mechanically
6. ask for explicit user confirmation
7. apply only the approved change

This keeps `check for updates` fundamentally different from `modify my system`.

## AI-Readable Breaking-Change Metadata

Owlenshell should dogfood the behavior it wants from upstream projects.

When Owlenshell itself introduces a breaking change, provide concise human-readable and machine-consumable migration information. A possible repository layout is:

```text
.release/
├── update.json
├── migrations/
│   ├── 1.8-to-1.9.md
│   └── 1.9-to-2.0.md
└── llms.txt
```

The exact format is intentionally undecided. The important properties are:

- source and target versions are explicit
- affected interfaces/configuration are explicit
- replacement behavior is explicit
- semantic differences are documented
- mechanical vs manual migration is declared
- validation commands are documented where available
- instructions are concise enough for humans and structured enough for agents

Example migration entry:

```text
BREAKING: workspace dispatch API

Removed:
    dispatch workspace 3

Replacement:
    workspace.focus(id=3)

Affected:
- CLI
- IPC clients
- config bindings

Mechanical migration:
    yes

Semantic differences:
    none
```

A useful principle:

> **Breaking changes may require migration work. They should not require archaeology.**

## Stable Public Boundaries

This idea reinforces a broader Owlenshell/OwlWC architecture rule:

> **Scripting is a client of the API, never the API itself.**

Configuration, IPC, CLI, scripting, Settings and future agent/MCP surfaces should consume stable typed operations rather than turning a scripting language into the external protocol.

Changing a configuration or scripting language must not inherently require changing IPC clients or CLI semantics.

When a public interface really must break, prefer:

```text
deprecate
  -> diagnose affected local usage
  -> document machine-readable migration
  -> automatically prepare migration where unambiguous
  -> validate
  -> remove only after a defined compatibility window
```

Another useful project principle follows:

> **Don't make users discover breaking changes by experiencing breakage.**

## Ecosystem Opportunity

The format should not need to remain Owlenshell-specific.

If other projects eventually publish compatible update/migration metadata, Owlenshell can consume it directly. When they do not, an optional analyzer can fall back to conventional sources such as release notes and changelogs.

This is effectively a small **breaking-change feed**: an RSS-like machine-readable interface for compatibility and migration information, combined with local knowledge of what the user actually runs.

Owlenshell does not need ecosystem adoption for the idea to be useful. It can start by:

- exposing its own update state cleanly
- consuming package/upstream information where available
- dogfooding high-quality migration metadata for Owlenshell releases
- making the LLM/agent integration optional
- measuring whether the preflight actually catches relevant breakage without producing noise

## Scope Discipline

This is a post-v1 integration idea. The v1 requirement remains a reliable update indicator if one is part of the daily-driver shell experience; AI analysis, ecosystem metadata and automated migration must not delay the core shell.

The long-term success criterion is simple:

> **Turn "63 packages have updates" into "61 are routine, one migration can be prepared automatically, and one decision needs your attention."**
