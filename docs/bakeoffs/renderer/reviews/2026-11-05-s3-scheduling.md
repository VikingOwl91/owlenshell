# Independent S3 review

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/reviews/2026-11-05-s3-scheduling.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Read-only Sol reviewer `/root/s3_review`, 2026-11-05. Requirements: authorized S3 only, existing SCTK/Coordinator ownership, one presenter, correct redraw/deadline handling, private control transport, no benchmarks. The reviewer received the requirement, milestone, diff and test evidence, with full inherited task context; this was an independent author review, not a blinded review.

## Before functional execution

Runtime/control source reviewed against signed S2 base `dfa0db8e2aee504fdf1357eda4d56c88d33b2b1c`, subsequently preserved as signed `9b4c7d8f1c27588f7a2c0932ef35d26fcff04562`. No ownership, raw-handle or presenter-exclusivity blocker found. Native attachment callbacks run outside manager borrows. Pipe validates inherited descriptor type, ownership, access mode, nonblocking flag and device/inode; there is no file fallback. Timers have cancellation/token/ticket guards; egui callbacks retain no native handles. The reviewer checked pinned calloop/Slint/egui behavior and independently reran the 21 existing harness tests.

- **LOW L1 — ignored checked clock-update Result:** both new calls now propagate errors. Parity tests passed again (24 library, 4 binary).
- **HIGH H1 — false idle acceptance:** the first oracle could accept pending-frame stalls or repeated loop work without submissions. Before any scheduling run, it was replaced with a quiescence check requiring the latest current-generation submission's matching frame callback and a final deadline-free blocked wait. Both ten-second windows reject loop work/wakes, redraw requests, deadline activity and submissions/callbacks; actual monotonic boundaries are recorded. Negative tests reject stalls, wrong generations, prior callbacks and wake/redraw storms. Actual retired tokens are excluded only after their destruction receipt.
- Retired cross-thread callback testing now uses the actual retirement boundary and matching pass/thread rejection. A missed bounded injection window is UNVERIFIED, never fabricated PASS or a stale-callback defect.

The reviewer independently reran all six revised S3 offline tests: PASS. **Pre-execution verdict: GO for private functional validation, no remaining BLOCKER/HIGH.** This is not final S3 acceptance or production approval.

## Boundaries and observations

The reviewer independently read both private EOF runs: COMPLETE, exit0, exact control binding/EOF, ordered native cleanup, egui GPU release before native destruction, instances_remaining0 and empty cleanup errors. All six recorded owned PIDs were absent during review. The reviewer did not execute candidates.

Slint timer/animation advancement follows native input dispatch in the existing calloop integration. The current bar/calendar fixture has no animation/transition blocks; animation freshness after long idle is not established and no generalized dispatch refactor is justified here. Direct signal-driven graceful Rust cleanup and server/GPU destruction completion remain UNVERIFIED. Client submission is not compositor/display presentation. Existing S2 gaps and historical NO-GOs remain.

Final evidence review will be appended after functional execution; no acceptance conclusion is inferred from the pre-execution review.

## Tightened offline reader

**MEDIUM M1 — omitted Core redraw traces:** the first reader parsed JSON redraw markers but not the existing Slint toolkit tracing marker. Both actual idle windows per renderer were independently checked against full raw logs and contain no hidden/trailing records: Slint wait131→next132 post-window pointer enter and final674→next675 quit wake; egui1493→1494 pointer enter and final7595→7596 quit wake. The observed idle findings remain supported. Before final handoff, the reader was tightened to normalize those Core redraw traces and reject a trailing redraw after a quiescent wait. Seven offline tests independently passed, and all four preserved windows plus both clock chains passed replay. No candidate was rerun; execution harness and later reader revisions are distinguished.

The offline evidence audit also now rejects Python optimization (which would disable assertions) and verifies the declared execution source bytes against local Git separately from the amended analysis-reader hash. These are evidence-reader corrections, not new candidate instrumentation or weakened criteria.

## Final evidence verdict

**PASS for the recorded S3 contracts; no unresolved BLOCKER, HIGH or MEDIUM findings.** The reviewer independently ran the offline audit over 394 preserved artifacts/eight functional sessions, verified exact execution-source and amended-reader bindings, balanced native lifetimes, four ten-second windows, five real domain ticks per renderer, cue pixels and stale callback rejection. Optimized Python failed closed as expected. The reviewer also reran the built 24library+4binary Rust tests and21+7 Python tests without launching candidates.

Both S2 regressions retain 87PASS/0FAIL/4UNVERIFIED, exact 4Toggle+4Hide actions and User-Open Focus. All EOF/failure cases have expected exits, cancellation and empty cleanup errors; recorded owned identities/groups were absent in independent metadata scans. The parent scan covers 122 identities. The original checkout and pre-existing handoff hash remain unchanged. Manual inspection of both timer-on captures corroborates the full visible calendar, clock 12:35:01 and expected pink cues; it is not full-scene equivalence or alpha-buffer proof.

Finite work differs: Slint has ten earliest-timer firings for five domain ticks; egui has pending/early/settling submissions before its single renderer cue transition. Quiescence is established only in the recorded windows, not as universal storm freedom or minimum repaint counts. Direct signal-driven graceful Rust teardown, server/GPU release completion, animation fidelity and prior S2 gaps remain UNVERIFIED. The final documentation filename NIT was corrected to `focus-observations.json`.

Runtime source: signed 9b4c7d8f1c27588f7a2c0932ef35d26fcff04562. Execution oracle: signed a1c462bccb935ba5f02452b7cbcee6dff693eea8. Tightened offline reader: signed 9b6f8f9fdf0c0be56e69f36996a8d142707685f2. Build cwd was the modified/tmp source snapshot; actual drivers loaded the durable branch source. Provenance distinguishes both. No candidate rerun accompanied the offline-reader correction. No S4–S6, benchmark, production or renderer-selection authorization follows.
