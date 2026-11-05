# OwlenShell portability requirements

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/PORTABILITY-REQUIREMENTS.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](bakeoffs/renderer/SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](bakeoffs/renderer/provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

2026-11-05. Architectural requirements adopted by the maintainer with [ADR-0006][adr6], not a compatibility certification. Initial target: **Linux with Wayland**, using Slint's software/`wl_shm` reference behind OwlenShell-owned SCTK surfaces. This document supplies constraints for a future separately approved base-planning phase; it authorizes no implementation or validation run. [Decision record][decision], [short planning handoff][handoff].

## Linux distributions

Target Arch Linux, Ubuntu, Debian, Fedora, NixOS and other compatible Linux distributions where dependencies and required Wayland capabilities are available. **Full cross-distribution support is UNVERIFIED**, including certification of Arch as a supported distribution rather than only the recorded test-host configuration.

Core architecture must not assume one distribution's package manager, package names, filesystem layout, service manager/session services or font installation paths. Packaging, dynamic/runtime dependencies, build-tool versions, assets/font discovery, configuration/state/runtime directories and optional service availability are explicit compatibility concerns. The experiment's absolute build/font/runtime paths are provenance bindings, not product filesystem requirements. Distribution-specific integration belongs in explicit packaging/integration policy, not implicit renderer/domain behavior. [Recorded fixture and assets][fixture], [post-S6 limitations][assessment].

| Distribution target | Current support evidence status |
|---|---|
| Arch Linux |UNVERIFIED as a complete distribution-support claim; scoped evidence applies only to recorded host/dependency versions |
| Ubuntu / Debian |UNVERIFIED |
| Fedora |UNVERIFIED |
| NixOS |UNVERIFIED; package/store and runtime discovery assumptions must be addressed explicitly |
| Other compatible Linux distributions |UNVERIFIED; require declared dependencies/capabilities and validation |

## Wayland compositors and capabilities

Target Niri, Hyprland, Sway, SwayFX and other compositors providing the required capabilities. Do not claim universal Wayland compatibility or identical layer-shell, focus, output, workspace or window-management behavior.

Keep three concerns separate, consistent with [ADR-0005][adr5]:

1. Common Wayland protocol functionality and Core surface/input/output lifecycle.
2. Narrow compositor-specific IPC/integrations that enrich common state.
3. Optional features whose availability and behavior depend on compositor capabilities.

Planning must distinguish mandatory startup/rendering prerequisites from optional enrichment. Missing required capabilities must produce explicit unsupported/error behavior. Missing optional services, workspace/window APIs or other optional capabilities must disable or reduce the affected feature explicitly and predictably, preserving basic shell operation where its prerequisites are satisfied. Do not silently substitute an ordinary desktop window for a required layer-shell surface. Exact minimum protocols/versions, capability discovery and fallback UX remain planning questions, not implemented mechanisms claimed here.

| Target / observation | Current evidence boundary |
|---|---|
| Niri |Scoped single-output S1–S4A evidence on the pinned nested version; multi-output **BLOCKED by the tested private environment**, not a Slint/egui failure |
| SwayFX |Scoped direct private headless S5/S6 two-output and lifecycle passes; overall coverage PARTIAL, not general version/hardware certification |
| Sway |Used as a private nested host; that role does not establish direct OwlenShell product acceptance across Sway versions |
| Hyprland |UNVERIFIED for this S1–S6 product-support claim |
| Other compatible Wayland compositors |UNVERIFIED; capability availability and policy must be observed, not presumed |

Focus-on-map under the adopted User-Open Focus contract is permitted after explicit clock activation; compositor focus grant is not universally promised. Workspace/window control and placement enrichment must not become hidden dependencies for unrelated basic UI. [S2][s2], [S5 Niri capability][niri], [SwayFX S5][s5], [S6][s6].

## Hardware and rendering modes

OwlenShell must not require a particular GPU or CPU vendor. Target AMD GPUs/APUs, Intel integrated/discrete GPUs, NVIDIA GPUs, systems without usable GPU acceleration where a supported CPU-rendering/display path is available, and different supported CPU architectures subject to dependency/build validation. No complete CPU-architecture or hardware-support matrix is validated by the current records.

A functional native CPU/software client-rendering baseline is required. Basic shell functionality must not require GPU acceleration. The initial Slint path supplies a concrete software renderer with Core `wl_shm` buffers and no client Vulkan-adapter requirement. The Wayland compositor/session must independently have a usable display/rendering path; the current tests do not prove a wholly GPU-free host. [Software path][adr6], [recorded S4A buffers][s4].

| Mode | Meaning / baseline obligation | Current evidence |
|---|---|---|
| Native software rendering |CPU paints UI into CPU-backed Wayland buffers such as `wl_shm`; initial required client baseline |Scoped Slint software/shm operation observed; CPU-only **host** coverage remains UNVERIFIED |
| Software implementation of a GPU API |For example lavapipe implements Vulkan on CPU; still requires that GPU-API/driver stack |Distinct from native CPU/shm presentation; not a substitute for its requirement and not a validated fallback here |
| Hardware-accelerated GPU rendering |A GPU API and capable device/driver execute rendering |Native egui Vulkan/RADV worked in its recorded environment; no vendor-independent or all-driver certification |

AMD configuration evidence does not certify every AMD device/driver. Intel/NVIDIA, CPU-only host operation and additional CPU architecture/ABI coverage remain **UNVERIFIED beyond the tested configuration**. Shared-memory pixel format, scale, fonts, dependency availability, toolchain and instruction/ABI assumptions must be explicit in future support validation. GPU acceleration can be introduced or used when supported, without making it a hidden prerequisite for the required basic client path.

## Native ownership and integration constraints

Evidence labels refer to the isolated functional fixture. They do not imply the ordinary production entry already incorporates all fixture behavior. Core here is the existing runtime/domain/platform responsibility, not a new mandatory crate or generic rendering framework. [Source/coupling assessment][assessment].

| Constraint for the initial base | Existing evidence versus future obligation |
|---|---|
| I1. Core/SCTK owns native connection, dispatch, outputs/identities, layer policy, surfaces, placement and destruction |Scoped integration demonstrated with both adapters. Preserve this responsibility in subsequent base design. |
| I2. UI work is subordinate to native lifecycle; invalidate retiring epochs before resource/native teardown |S6 verifies ordering in its private SwayFX scenarios. Other dispatch orderings and production composition still need acceptance. |
| I3. Exactly one presenter per surface generation; no competing shm/wgpu submissions |Concrete software/native slots and runtime evidence exist. Preserve the invariant for optional acceleration; no generic GPU framework is required. |
| I4. Reject stale work using surface-generation tokens and UI-registration/incarnation identity |Scoped tests distinguish destroyed epochs from retained logical UI. Native-only recreation may preserve valid queued logical intent; removed UI registrations must not affect replacements. Do not collapse these identities. |
| I5. Domain state is separate from renderer UI state |Real controller state survives fixture UI/native retirement. Retain that separation in real module/configuration composition; the fixture is not a complete production module system. |
| I6. Compositor-specific features use narrow capability-aware integrations |Accepted design requirement/ADR-0005, not blanket validation of every compositor adapter or behavior. |
| I7. GPU acceleration is optional for basic shell functionality |Slint software/shm is a demonstrated client baseline. CPU-only hosts and broad vendor/architecture coverage remain UNVERIFIED. |
| I8. Missing optional services/capabilities fail or degrade explicitly and predictably |Future product requirement. Complete optional-service discovery/fallback behavior has not been validated by the mock domain fixture. Mandatory capability failures must remain explicit. |
| I9. Avoid premature general-purpose renderer-plugin abstractions |Design constraint. The tested narrow UI boundary does not require a universal backend API or runtime selection system. |
| I10. Preserve evidence and a practical path to a later renderer decision |Keep domain/native policy separate, document integration costs and retain experiments. This does not promise cost-free replacement or require speculative multi-renderer production runtime. |

Core retains scheduling authority through the event loop and renderer deadline hooks; Slint-specific timer mechanics stay in the integration. Persistence concerns logical intent, not ephemeral surface tokens. Actual composition/production migration remains future work. [S1][s1], [S3][s3], [S5][s5], [S6][s6], [contracts][ui].

## Software-renderer visual and feature limits

S4A compatibility uses integer1/2 buffers with compositor downscaling at fractional output scales. Downsampling softness, font stem/weight/antialiasing differences and incomplete per-glyph/fallback identification remain recorded; native fractional-buffer sharpness is not established. Composed captures are not actual alpha-buffer or physical presentation proof. Broad text/locales, accessibility and UI-feature coverage are not inferred from the bar/calendar. [S4A][s4].

The custom Slint adapter fixes a pre-existing preferred-size/property callback defect while retaining software rendering; it does not reproduce the earlier private developer full-speed rendering-metrics mode. That scoped difference is documented, not evidence of a product performance advantage. [Calendar closure][calendar]. Runtime `slint-interpreter`, UI hot-reload, theme reload and state migration are **UNVERIFIED future possibilities**, not implemented product features or proven reasons for selection.

## Outstanding validation register

| Topic | Preserved status and required interpretation |
|---|---|
| Niri multi-output |**BLOCKED** by the tested private nested environment; no renderer-failure inference |
| Keyboard-focused output removal / restoration |UNVERIFIED |
| Controlled native-wire races / exact boundary interleavings |UNVERIFIED; observed old renderer Context wake rejection is narrower evidence |
| Physical monitor hotplug |UNVERIFIED; synthetic output-global removal/rebirth is not physical connector validation |
| Native fractional-buffer rendering |UNVERIFIED / not implemented in S4A; fractional-output compatibility is separately scoped PASS |
| Full cross-distribution compatibility |UNVERIFIED |
| Intel/NVIDIA, CPU-only host and other CPU architecture coverage |UNVERIFIED beyond the recorded configuration; software client path alone does not establish host coverage |
| Runtime Slint UI hot-reload / theme reload / state migration |UNVERIFIED |
| Complete optional-service/capability degradation |UNVERIFIED future requirement |
| Production integration and acceptance |**NOT PERFORMED**; Gate P has not passed |
| Decision-grade performance comparison |**NOT AVAILABLE**; prior CPU/memory results permanently EXPLORATORY — NON-DECISION-GRADE |
| Duplicate native pointer leave |Protocol-level observations remain PARTIAL; scoped semantic cancellation is idempotent and action checks pass. No raw events were suppressed; root cause is not universally established. |

Both renderers retain their scoped functional passes and remaining gaps. No candidate status or historical NO-GO is rewritten by the selection. [S6 scope][s6], [post-S6 assessment][assessment], [preserved observations][exploratory].

## OS scope and subsequent authority

Cross-distribution Linux support does **not** imply Windows, macOS, BSD or X11 support. Those targets are outside the initial architectural commitment unless separately approved.

These requirements are inputs, not an implementation plan. They prescribe neither packages/services to install nor a new compositor/renderer infrastructure. Actual support claims need explicit version/hardware/capability evidence. Stop after this documentation; await separate maintainer approval for base-architecture planning, implementation, migration or new execution. [Handoff][handoff].

[adr6]: ADR/0006-renderer-selection.md
[decision]: bakeoffs/renderer/decisions/2026-11-05-initial-slint-selection.md
[handoff]: bakeoffs/renderer/SLINT-BASE-PLANNING-HANDOFF.md
[adr5]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-eea96193892d525f
[assessment]: bakeoffs/renderer/POST-S6-DECISION-READINESS.md
[fixture]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-86bbf6943147c0e4
[s1]: bakeoffs/renderer/results/2026-11-04-s1-functional-parity/README.md
[s2]: bakeoffs/renderer/results/2026-11-05-s2-user-open-focus/README.md
[s3]: bakeoffs/renderer/results/2026-11-05-s3-scheduling/README.md
[s4]: bakeoffs/renderer/results/2026-11-05-s4-scaling/README.md
[s5]: bakeoffs/renderer/results/2026-11-05-s5-multi-output/README.md
[s6]: bakeoffs/renderer/results/2026-11-05-s6-lifecycle/README.md
[niri]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-152e70ea128027c3
[calendar]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-e7c0480d239b7289
[ui]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-e2a9690cc59100a9
[exploratory]: bakeoffs/renderer/SOURCE-REFERENCES.md#source-650d9c951033d2ac
