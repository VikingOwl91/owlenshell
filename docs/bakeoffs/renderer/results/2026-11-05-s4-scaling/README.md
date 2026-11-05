# S4A output scaling compatibility

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/results/2026-11-05-s4-scaling/README.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

**PASS within the recorded S4A compatibility scope for Slint and native egui.** Functional validation only — no CPU/RSS/PSS/FPS benchmark, renderer ranking, production integration or selection. Base: signed `af5dc0dbf2b4910a8554e9ab5b190e02d8d6eff5`.

[Contract](../../SOURCE-REFERENCES.md#source-223d6169fff48877), [maintainer authorization](../../SOURCE-REFERENCES.md#source-1c5f8197d9a09dd5), [independent review](../../reviews/2026-11-05-s4-scaling.md). Core retains SCTK surfaces, roles, output tracking, generations, input routing and the sole presenter slot. Root production code/manifests, existing collector and S1–S3 evidence are unchanged. No wayapp, fractional-scale/viewport implementation, general plugin/scheduler infrastructure or private Niri diagnostic is added.

## Per-renderer, per-scale acceptance

| Effective output scale | Slint functional / visual | egui functional / visual | Observed mode |
|---|---|---|---|
|1.00|PASS / PASS|PASS / PASS|integer1 buffers, integer output|
|1.25|PASS / PASS|PASS / PASS|integer2 buffers, compositor downscaling|
|1.50|PASS / PASS|PASS / PASS|integer2 buffers, compositor downscaling|
|1.75|PASS / PASS|PASS / PASS|integer2 buffers, compositor downscaling|
|2.00|PASS / PASS|PASS / PASS|integer2 buffers, integer output|

Functional evidence: `runs/{slint,egui}-scaling-03/`, signed source `d818677967e3b2e9c25a52a0bf20e60d34676677`, candidate SHA256 `df1ff3d702f603792c0df178b4fe97838f1b24ad9858e2d4784d3867be5828aa`. Each has211PASS/0FAIL/3UNVERIFIED assertions. Visual full-output closure: separately identified `runs/{slint,egui}-visual-01/`, same candidate binary, signed harness `1e657fd9eb3442068259a03b55410fdf8a1c15af`,41PASS/0FAIL/3UNVERIFIED each. Counts describe different contracts, not scores. The three grouped UNVERIFIED entries preserve native fractional buffers, display/alpha authenticity and prior S2/S5/S6 scope boundaries.

Both series use the private headless Sway → installed unpatched Niri26.04(8ed0da4), physical2560x1440@60Hz, hostscale1, one winit output and its recorded Flipped180 transform. Actual host mode, PID-selected fullscreen container and Niri state are verified; no app-ID guess or productive socket is used. Fractional output settings are directly observed through private IPC; clients receive integer wl_output scale and xdg-output logical-size events. Advertisement of fractional globals is not use.

## Actual geometry and buffer evidence

The following values were independently observed for both candidates; the shared test never assumes their renderer internals are the same.

| Output scale | Niri IPC logical extent | Client xdg-output extent | Bar configured logical size | Bar buffer | Calendar buffer | wl_surface buffer scale |
|---|---|---|---|---|---|---:|
|1.00|2560x1440|2560x1440|2560x32|2560x32|320x320|1|
|1.25|2048x1152|2048x1152|2048x32|4096x64|640x640|2|
|1.50|1706x960|1707x960|1707x32|3414x64|640x640|2|
|1.75|1462x822|1463x823|1463x32|2926x64|640x640|2|
|2.00|1280x720|1280x720|1280x32|2560x64|640x640|2|

Calendar configured size remains320x320; TopRight margins top40/right24 follow the bar's exclusive32. Complete placement/bounds evidence accompanies snapshots and captures. IPC truncation and client rounding are separate APIs: the client events and layer configure/ack agree, while IPC can differ by one logical pixel. Nearest-round validation is scoped to this pinned fixture, not a universal fractional-buffer allocation rule. The originally hypothesized client1463x823 at1.75 is now directly observed in both runs.

Each step stores actual output, Coordinator/host state, native token, configure/ack serial/line, set_buffer_scale, actual wl_buffer constructor dimensions, attach and commit. Slint uses wl_shm; egui uses linux-dmabuf through its native wgpu presenter. Buffer dimensions exactly equal acknowledged logical dimensions × committed integer buffer scale. The reader resets role/object lifetimes on reuse, rejects snapshot substitution, checks committed null attachments and preserves already committed dimensions after a client buffer object is destroyed. Old acknowledged-size frames while a new configure is pending are permitted. Scale-only changes need no new logical configure; fractional→fractional integer2 calendar content may be reused.

The Slint wrapper forwards existing Core configure/ScaleFactorChanged/physical-size handling to the software renderer. Egui explicitly stores Core's integer scale, derives logical extents once, sets ROOT native PPP without changing zoom, uses actual FullOutput PPP for tessellation/ScreenDescriptor, and leaves native logical input coordinates untouched. Neither path implements native fractional-client buffers. **Fractional-output compatibility is verified; native fractional-buffer support is not implemented/UNVERIFIED.**

## Input, state and lifecycle

At every ascending scale, a real native Enter→press/release without preceding Motion activates the clock once. Four logical popup corners deliver correctly; outside-close(302,30) causes no action, inside-close(302,26) causes one Hide; reopen gets a newer generation. Actual wheel±1 calls the real calendar controller, submits the changed state and restores November after December. Both full runs record19 intended lifecycle actions(14Toggle+5Hide), ten domain scroll changes and eleven balanced native lifetimes. All other timer/output work causes no duplicate lifecycle action.

The thirteen live phases are1→1.25→1.5→1.75→2→1.75→1.5→1.25→1→1.25→1.75→1.25→1. Calendar generation/UI registration and intended month remain stable during the eight later live transitions. Configure/resize, backpressure and resource retirement use the unchanged Core path. Client GPU drop precedes native destruction; server/driver completion is not thereby established.

Requested and actual native positions/errors are preserved. The fixed pre-execution bound is1+1/256 logical pixels for cross-API rounding/normalization uncertainty; actual maximum error is **0.0** for the tested points in each current full run ([data](../../SOURCE-REFERENCES.md#source-71c135b498d0aed4)). This is not a claim about every point or every compositor. The fixed boundary probes and tolerance were never moved or widened to make a run pass.

Adopted User-Open Focus remains: explicit clock activation may be focused immediately by the compositor; grant is observed, not promised universally. Scaled runs correlate actual focus, but do not establish complete keyboard parity at all scales. Separate new-binary scale1 S2 runs retain87PASS/0FAIL/4UNVERIFIED each. New-binary S3 runs retain Slint19PASS/3UNVERIFIED/1NOT APPLICABLE and egui23PASS/3UNVERIFIED, including both ten-second quiescent windows and real deadlines. No prior S2/S3 assertion or historical NO-GO is relabeled.

## Visual acceptance and capture limitations

All26 supplemental full-output captures are2560x1440 and pass the unchanged2physical-pixel cyan calendar boundary check; maximum observed edge error is1px ([bounds](../../SOURCE-REFERENCES.md#source-0cb8ef28a7e6952f)). Parent inspection of all ten first-scale calendar crops and clock rows verifies complete November2026, all1..30 grid glyphs, close and12:34:56 clock, with no observed title/grid/control clipping. Exact unresized ROI crops and source hashes/bounds are preserved in [visual-crops.json](../../SOURCE-REFERENCES.md#source-f08e9e2f33de9c38), [clock-crops.json](../../SOURCE-REFERENCES.md#source-aba7dae01d613112) and [manual inspection](../../SOURCE-REFERENCES.md#source-9c0f3f87cfafbd00). The reviewer independently inspected all five original scales, both new full1.50 captures and then all ten supplemental calendar ROIs and both five-row clock strips. This is bounded visible-content acceptance, not pixel-perfect scene equivalence.

Fractional captures show downsampling/AA softness; font stem/weight/rasterization differences remain visible. There is no native-sharpness or equal-rasterizer claim, nor authenticated per-glyph font/fallback identity. Captures are composed RGB evidence, not alpha-buffer or physical-display/presentation feedback. Text in other months/locales is not exhaustively visually inspected.

Original03 default-grim1.50images are2560x1439 and **remain incomplete visual evidence**. Candidate-free probes receive2560x1440 via screencopy, yet default/output-only grim writes1439rows. Exact internal inference is unproven. Explicit `-o winit -s <verified effective scale>` yields full-sized images; every supplement records argv/scale/size and fails on any dimension mismatch. No old PNG was resized, repaired or promoted. Tool resampling can affect composed pixels; all sharpness observations are qualitative. The supplement has only one actual clock activation and retains one popup generation; its hover/input history differs from the full-input run and it does not replace that run.

## Preserved negative findings and remediation

| Series | Recorded outcome | Established cause / prospective correction |
|---|---|---|
|Both scaling01|FAILED initial configure/submission timeout|Initial valid frame preceded phase0; reapplying scale1 needed no redundant repaint. Only startup may reuse it; later changed-geometry freshness gates remain.|
|egui scaling02|FAILED outside-close-retained|Correct native(302,30) delivery; egui0.35 default5logical-pixel nearby-hit radius selected close. Controlled actual-scene PPP1/2 A/B tests prove default outside activation, zero-radius rejection and inside26 activation. Prototype Context now uses zero radius for both themes; no focus/native policy change.|
|Slint scaling02|FAILED at1.50 geometry|Client1707x960, configure1707x32 and3414x64 buffer were coherent; the harness wrongly equated them to IPC1706. It now records actual client geometry separately.|
|03 default1.50captures|Full-output visual gate incomplete|Capture postprocessing gap, corrected only in a distinct reviewed visual series; fixed pixel/dimension criteria unchanged.|

Earlier504 candidate binary and all failed01/02 receipts, raw native events and state snapshots remain preserved. The zero-radius mouse policy is prototype-only; production accessibility hit padding remains a maintainer decision. Build/test attempts include pinned-API E0599 corrections and restricted-socket EPERM followed by successful disposable validation. They are not renderer performance or compatibility measurements.

The first candidate-free12step preflight's exact uncommitted source snapshot was not preserved: its source binding is explicitly UNVERIFIED, never assigned a later clean13step revision. Clean signed environment02 verifies the final fixture; capture probes preserve exact driver snapshots and helper bindings. Source and binary identity limitations are not silently waived.

## Reproduction, review and preservation

[Provenance](../../SOURCE-REFERENCES.md#source-bdfd604b44e7e2a4) records signed revisions, exact source-file hashes, dependency package/lock versions, toolchain, build flags, binary/fixture hashes and local protocol references. Per-run environment records compositor/tool/config hashes and whitelisted runtime variables. [Artifact manifest](../../SOURCE-REFERENCES.md#source-647783d25b137c20) preserves raw/stored hashes and lengths; logs are lossless gzip(mtime0). Generated binaries, caches and sockets are excluded. Use fresh private paths and the recorded pinned candidate/fixture when reproducing; original historical executable paths may now contain later binaries, so match hashes or use the preserved binary location.

Tests actually run: **132 Core,33 parity(29library+4pipe),49 harness and4 input-fixture PASS**, with formatting/source diff checks. Raw test stdout retains its trailing blank lines and is excluded only from source whitespace checks. The standalone offline reader `python3 docs/bakeoffs/renderer/results/2026-11-05-s4-scaling/audit.py` launches no candidate, rejects Python optimization, checks source/reader/binary/artifact bindings and replays wire/state/geometry/idle evidence. See [offline audit](../../SOURCE-REFERENCES.md#source-8a52d8675b3d15d5) and negative controls. It deliberately retains failed runs rather than counting them as successful.

All18 private sessions have empty cleanup-error receipts, including four candidate-free probes and four failed full-input attempts. A UID-scoped identity/group/session scan found no survivors among285 tracked identities ([cleanup audit](../../SOURCE-REFERENCES.md#source-c326a04f30c1260e)). Successful clients exit0 with native instances0; forced cleanup after failed harness assertions is not proof of direct-signal graceful Rust teardown. Original checkout/handoff and historical evidence are preserved.

**Independent verdict: PASS for recorded S4A compatibility, with explicit limitations.** Gate R remains BLOCKED; Gate P incomplete. Native fractional APIs, physical connector behavior, other transforms/compositors, keyboard at every scale, device recovery, direct-signal Rust teardown, server/GPU release completion and prior four S2 gaps remain UNVERIFIED/outside this slice. No S5/S6, benchmark or production work follows. ADR-0006 remains Proposed and the private Niri diagnostic disabled.

Renderer and architecture selection remain exclusively with the maintainers. No winner has been selected.
