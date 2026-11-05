# Independent S1 functional-parity implementation review

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/reviews/2026-11-04-s1-functional-parity.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Completed2026-11-05 Europe/Berlin; filename retains run-start2026-11-04. Read-only Sol reviewer `/root/s1_review`, independent from Core/Slint implementation, egui implementation and candidate execution. Baseline `7b83576ffd7e7bd028ae4270937ac06167ee373c` and approved S1/FP1 plan. [Final report and evidence](../results/2026-11-04-s1-functional-parity/README.md).

**Verdict: PASS, scoped to authorized S1 action/state/lifecycle on one output at scale1. No open BLOCKER, HIGH, MEDIUM or LOW correctness/safety findings.** No later-slice, production, benchmark or renderer-selection authorization follows.

## Source and ownership review

Reviewed frozen implementation before candidate launch, then the complete actual runtime evidence and final report. Confirmed actual Coordinator/controller state, Core SCTK/SurfaceManager/generation ownership, per-host presenters, no adapter-owned Coordinator or Rc cycle, lazy own-token activation, no Core shm competition for native slots and retained raw-handle lifetime. Shared pump-before-action-drain fence avoids reentrancy and aligns the existing header-drag placement phase. UI re-registration retires through Coordinator before replacing its adapter; logical month persists independently of native/UI epochs. The bounded64 queue rejects old UI incarnations and makes overflow terminal rather than dropping/coalescing Toggle. Root changes stay limited to UTC clock fixture formatting and propagating fallible render errors; no production UI/manifest migration.

## Findings and verified remediation

| Severity | Finding | Closure |
|---|---|---|
| MEDIUM | Driver initially checked action count only after slow fixture cleanup and used15s waits, potentially accepting late actions beyond the approved guard. | Action delta1 now required within5s before stopping the fixture; post-cleanup duplicate check retained. Snapshot/action default5s, startup/layers10s, injector exit separately10s. No timing score. |
| LOW | GPU readiness could use configured extent without a submitted frame for the current host generation. | Exact host+generation present-submitted marker required; regression rejects another host or old epoch. |
| LOW | Output lookup did not explicitly reject additional outputs before normalizing OutputIds. | Sole winit preflight mandatory; regression rejects another output/preexisting layer. |

A suggested close-font mismatch was independently refuted: source already specifies weight700; no source/tolerance change was made. Day-weight mapping and exact font/raster limitations are accurately retained in the report.

## Independent verification

- Exact candidate SHA-256: `c878958ff15a92e9b6af81cdb3fa064c7d16900828d13d309df8ef463829465c`; both expected font byte payloads are embedded. Per-glyph runtime association is not proved.
- Executed exact built offline tests:11 focused parity,3 root unit,14 Coordinator,17 RenderBridge, and9 Python controls PASS. Reviewed parent's independently executed full123-test Core passing log. No candidate/compositor or performance execution by reviewer.
- Both positive runs:25 physical click sequences each with exactly one accepted controller action;30 state labels compare exactly, including SavedPlacement/placement decisions/native geometry; private placements files byte-identical. Bar generation1 stable,16 popup generations per renderer, old UI intent rejected once.
- Inspected all16 positive PNGs. Full frozen clock, November/December month/title/grid/close visible; December persists through reopen/native/UI recreation and returns to November only on explicit logical reset. No whole-image/pixel/alpha equivalence claimed.
- Independently checked17 actual native lifetimes per renderer: configure/ack precedes first buffer attachment and all roles/surfaces destroy. GPU attached buffers and own-token GPU release precede corresponding native role/surface destruction; GPU run has no shm pools. Slint uses real software/shm path.
- Both targeted calendar-failure controls: successful bar first, one accepted Clock action, one expected calendar error, no calendar frame, terminal exit1, two native role/surface lifetimes destroyed and cleanup errors[]. Slint synthetic first-paint wrapper versus egui post-device initialization are distinct fault mechanisms.
- All positive28+28 and negative4+4 recorded process leaders/groups absent after cleanup. Existing trusted-process/guardian limitations remain; no hostile session escape or uninterruptible-I/O guarantee.
- Packaged audit independently rerun PASS;26/26 source bindings and307/307 evidence-file hashes verified at final freeze. Lossless compressed raw log checksum/length confirmed. Final signed-commit copy/signature verification belongs to the parent.

## Limits and final claims

Bounded composited RGB difference was independently reproduced: at(1000,380), Slint(3,6,10), egui(2,5,9). Default egui-wgpu dithering is enabled and is plausible, not isolated pixel causality. Slint day cells request numeric weight600; egui selects its registered bold file without a numeric weight parameter. Do not claim numeric requested-weight/per-glyph/pixel equivalence. No settings or tolerance were changed after runs.

The report also preserves the no-motion header-close persistence side effect relative to pristine inline Slint callbacks. Both S1 variants produce identical current state/placement; this is not proof of every pristine wiring side effect. S2–S6, keyboard/scroll/focus, other scales/outputs, recovery, deadlines, full visual/alpha acceptance, resource/performance benchmarks and full production integration remain unverified.

No source edits or candidate/compositor execution by reviewer. One Python import generated a bytecode cache, which the parent removed; reviewed source remained unchanged. Historical NO-GOs stand, ADR-0006 stays Proposed, private Niri diagnostic disabled. Renderer selection remains a maintainer decision. No winner has been selected.
