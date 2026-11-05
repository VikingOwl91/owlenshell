# OwlenShell

OwlenShell is an experimental, modular Linux/Wayland desktop shell written in Rust. Its initial renderer is Slint, with native Wayland surfaces owned by an SCTK-based Core.

> **Status: Early Architecture & Prototyping**
>
> OwlenShell is currently in its early architecture and prototyping phase. The project is exploring a modular Linux/Wayland desktop shell built around atomic surfaces, independent hosts and widgets, and clearly separated system services.
>
> Initial Wayland and renderer experiments have demonstrated selected architectural concepts. Slint has been selected as the initial renderer, but OwlenShell is not yet a complete or production-ready desktop shell.
>
> Most planned shell features and broader compatibility goals remain future work.

[Architecture](docs/ARCHITECTURE.md) · [Renderer decision](docs/ADR/0006-renderer-selection.md) · [Evidence](docs/bakeoffs/renderer/README.md) · [Portability](docs/PORTABILITY-REQUIREMENTS.md) · [GPL-3.0-only](LICENSE)

## Vision and design principles

A widget's content should be independent of where it lives. A clock can belong to a bar; a calendar can have its own popup host. Hosts manage presentation and placement without giving widget code ownership of native Wayland surfaces.

- **Atomic surfaces:** visible shell regions have appropriately sized native surfaces and input regions. A host can contain several widgets; every small widget does not need its own surface.
- **Core-owned lifecycle:** OwlenShell owns Wayland connections, output tracking, layer-shell policy, native surface generations and retirement. UI rendering follows that lifecycle.
- **Typed state and actions:** domain state, placement intent and commands stay separate from renderer-specific presentation. Recreating a surface should preserve its logical state.
- **Shared system integrations:** planned Rust service components provide Linux system state to independent widgets. Service logic belongs outside the UI toolkit.
- **Capability-aware portability:** common Wayland behavior forms the foundation. Compositor-specific functionality belongs behind narrow adapters, with explicit handling of missing capabilities.

These are design constraints and goals. Their full production implementation remains ahead.

## Architecture

The repository currently contains one Rust package with a custom Wayland platform and Slint UI components. It is not yet the complete service and extension architecture described in the design documents.

| Boundary | Responsibility and current status |
|---|---|
| Domain and Coordinator | Implemented prototype types for hosts, widgets, commands and composition; clock/calendar controllers and placement coordination |
| SCTK / Wayland platform | Implemented native connection, output tracking, layer surfaces, input dispatch and software-buffer presentation |
| Placement and persistence | Implemented placement resolution and stored placement intent; broader physical-topology behavior still needs validation |
| Slint UI | Existing declarative bar/calendar components and a custom software-rendering bridge |
| Compositor adapters | Existing boundary and limited metadata integration; broader workspace/window integration is planned |
| Linux system services | Planned audio, network, Bluetooth, power, media, notifications and tray integrations |
| Configuration, theming and extensions | Design work exists; a complete user-facing configuration/reload/extension system is planned |

[ADR-0006](docs/ADR/0006-renderer-selection.md) records the maintainer's selection of Slint as the initial Linux/Wayland renderer. The concrete baseline renders on the CPU and presents through `wl_shm`; basic client rendering does not require a Vulkan adapter. The compositor still needs its own usable rendering/display path. Native CPU rendering, software implementations of GPU APIs and hardware acceleration are distinct capabilities.

See the [architecture](docs/ARCHITECTURE.md), [atomic-surface model](docs/ATOMIC-SURFACES.md) and [configuration design](docs/CONFIGURATION.md) for the intended boundaries. Those documents include future requirements as well as existing concepts.

## Current progress

| Status | What it means here |
|---|---|
| **Implemented** | Rust domain and Coordinator foundations, output/placement/persistence logic, SCTK-owned surfaces, custom Slint software rendering, bar and clock/calendar prototype code, and focused tests |
| **Experimentally validated** | Separate Slint and native egui/wgpu experiments passed their declared S1–S6 functional scopes: composition, input, scheduling, scale compatibility, multi-output behavior and lifecycle/shutdown controls |
| **Planned** | A production-oriented base composition, modular system integrations, configurable shell components, portable packaging, broader theming and extension support |
| **Not yet validated** | Complete Gate P acceptance, broad distribution/compositor/hardware support, physical monitor hotplug, native fractional-buffer rendering, runtime Slint UI reload and full production integration |

The [curated renderer evidence](docs/bakeoffs/renderer/EVIDENCE-SUMMARY.md) preserves the scope and limitations of those experiments. Overall functional coverage remains **PARTIAL**. SwayFX multi-output results are compositor-specific; Niri multi-output remains **BLOCKED by the tested private environment**, rather than established as a renderer failure. Scale compatibility used integer-buffer/compositor scaling and does not establish native fractional-buffer support.

The experimental implementations and complete raw history are retained separately; the public snapshot contains reviewed narrative evidence and provenance, not a complete reproduction package. Both Slint and native egui demonstrated the tested ownership boundary. Slint's selection is a maintainer decision about the initial baseline, not a claim of universal superiority. Previous performance observations remain **EXPLORATORY — NON-DECISION-GRADE**; no decision-grade performance ranking exists.

OwlenShell is not ready to replace an existing desktop shell or panel setup. Gate P remains unpassed.

## Roadmap

The next phase is **base-architecture planning**, using the [Slint handoff](docs/bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md) and preserved evidence. Implementation and migration require their own bounded decisions.

- Define the first usable feature set, supported environments and production acceptance requirements.
- Plan how the tested ownership boundaries fit the real application composition.
- Build out shared Linux services, configuration and shell components in small validated steps.
- Expand visual, lifecycle, hardware and compositor validation before claiming support.

The [v1 scope](docs/V1.md), [production architecture plan](docs/PRODUCTION-ARCHITECTURE-PLAN.md) and [ideas](docs/IDEAS.md) describe longer-term direction. They are not release promises or lists of completed features.

## Portability and compatibility goals

The initial target is **Linux with Wayland**. Cross-distribution support does not imply Windows, macOS, BSD or X11 support.

| Area | Architectural targets; validation still required |
|---|---|
| Linux distributions | Arch Linux, Ubuntu/Debian, Fedora, NixOS and other compatible distributions |
| Compositors | Niri, Hyprland, Sway, SwayFX and others providing the required Wayland capabilities |
| Hardware | AMD, Intel and NVIDIA GPUs; supported CPU architectures without a vendor-specific requirement |
| Rendering | A functional native CPU/software baseline; optional acceleration where supported |

This is a target list, **not a production compatibility matrix**. Required protocols, dependencies, compositor policy, fonts and packaging must be checked for each supported combination. Optional capabilities should degrade explicitly without disabling unrelated shell functions. No wholly GPU-free host or broad CPU-only hardware coverage is certified by the current experiments. See the [portability requirements and gap register](docs/PORTABILITY-REQUIREMENTS.md).

## Development and building

This is a source-level prototype for contributors. There are no installation or session-replacement instructions here.

Use a Rust/Cargo toolchain capable of building the checked-in lockfile, a native linker and the development libraries required by the resolved dependencies. Distribution-specific package recipes and minimum supported toolchains have not been validated across the target distributions.

From the repository root:

```sh
cargo build --locked
cargo test --locked
```

These commands build the prototype and run the current automated tests. They do not establish visual correctness, compositor compatibility or production readiness. Initial dependency downloads require network access; an already populated Cargo cache can be used offline. The build verification environment and limits are recorded in [licensing and validation notes](docs/LICENSING.md#validation-scope).

Running the application creates real Wayland shell surfaces. Runtime investigation should use an explicitly isolated compositor environment; the curated evidence is not a runnable harness. Building or testing is not an instruction to replace a running desktop session.

## Documentation

- [Project identity](docs/IDENTITY.md), [architecture](docs/ARCHITECTURE.md) and [atomic surfaces](docs/ATOMIC-SURFACES.md)
- [Accepted initial renderer decision](docs/ADR/0006-renderer-selection.md) and [evidence framework](docs/ADR/0007-renderer-bakeoff-evidence-scope.md)
- [Curated bake-off index](docs/bakeoffs/renderer/README.md), [unranked summary](docs/bakeoffs/renderer/EVIDENCE-SUMMARY.md) and [provenance](docs/bakeoffs/renderer/provenance.json)
- [Portability requirements](docs/PORTABILITY-REQUIREMENTS.md) and [Slint base-planning handoff](docs/bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md)
- [Configuration design](docs/CONFIGURATION.md), [v1 direction](docs/V1.md) and [extension design](docs/EXTENSIONS.md)
- [License scope and validation notes](docs/LICENSING.md)

## Contributing

Early contributors are welcome. Small, documented investigations, tests, portability reports and clearly scoped improvements are especially useful while the base architecture is taking shape.

Before a substantial change, open an issue describing the problem, proposed scope and supporting evidence. Distinguish observations from assumptions, keep domain and native-surface ownership explicit, and avoid mixing production migration with documentation or exploratory work. Include the relevant build/test results and preserve negative findings.

Only contribute material you have the right to submit. Preserve third-party notices and identify any separately licensed material. Project contributions are intended under GPL-3.0-only unless an explicitly documented separate license applies; see [license scope](docs/LICENSING.md).

## License and acknowledgements

OwlenShell's project-authored code and documentation are licensed under the **GNU General Public License version 3 only**, SPDX **`GPL-3.0-only`**. See [LICENSE](LICENSE). Dependencies and separately licensed third-party material retain their own terms; the project license does not relicense them.

OwlenShell builds on [Slint](https://slint.dev/) and [Smithay Client Toolkit](https://github.com/Smithay/client-toolkit). [GPUi Shell](https://github.com/andre-brandao/gpui-shell) is a source of inspiration for Linux service integration and README presentation. OwlenShell is an independent project; this refresh copies no GPUi Shell prose, source code, artwork or screenshots. Any future direct reuse needs a separate licensing review.
