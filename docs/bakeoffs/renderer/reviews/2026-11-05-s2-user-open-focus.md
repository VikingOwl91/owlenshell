# Independent S2 User-Open Focus review

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/reviews/2026-11-05-s2-user-open-focus.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

2026-11-05 Europe/Berlin. Independent read-only Sol reviewer. Maintainer-approved base `9d73831b915445895f0eec61db5d3267a74bfa30`; tested signed harness `588ed633006337d62e76cdc44ba32d60188dc0f2`.

## Final verdict

**No outstanding findings. Scoped User-Open Focus validation PASS.** Historical FAIL/NO-GO evidence remains valid and unchanged. No full Gate R/Gate P, S3, production, benchmark or renderer-selection authorization follows.

The reviewer independently verified19 harness tests and five audit controls, recomputed the audit, and confirmed:

- Both runs have87PASS/0FAIL/4UNVERIFIED,8 intended alternating Toggle/Hide actions and6 native lifetimes destroyed.
- Seven in-run focus observations are distinct from the separately labeled post-run replacement-focus reconstruction. The latter uses actual pre-click native/Core/UI receipts and snapshot, hashes its endpoint, and confirms no new-generation press preceded it.
- Thirty-three canonical domain/placement/focus snapshots match; this is not scene/pixel or performance equivalence.
- Three harness hashes match the signed tested source,54 unchanged runtime input hashes and the reused binary hash match earlier source evidence.
- All343 historical S2/triage files are unchanged; compressed raw-log hashes/lengths verify.
- All100 recorded owned process identities/groups are absent; cleanup receipts have no errors.
- Collector, renderer/Core implementation and ADR-0006 remain unchanged.

The review inspected the maintainer product decision, exact assertion/contract diff, actual focus/key destination evidence, generation handling, active input cancellation and ordered teardown. No focus event suppression, new native policy mechanism or unrelated assertion relaxation was found.

## Pre-runtime findings and fixes

MEDIUM — UI generation metadata was not checked by the first observer draft. An egui UI receipt labeled generation6 could accompany native/Core generation7 and return GRANTED. Fixed by validating explicit UI generation and the exact native creation boundary. Slint lacks this UI field; the report retains that correlation limit rather than inventing metadata.

MEDIUM — A legitimate correlated Enter→Leave during the key probe could be labeled INCONSISTENT before the transition guard. Fixed with a correlated TRANSITIONED outcome and UNVERIFIED/BLOCKED dependent coverage. Missing/contradictory native/Core/UI evidence still fails. Regression controls exercise both cases; no candidate was run until review cleared these fixes.

A new regression exposed duplicate native-created boundaries being accepted through last-boundary selection. Requiring exactly one current-generation/current-surface boundary resolved it; final19 tests pass.

## Post-run instrumentation gap

The planned dedicated pre-click replacement label was not emitted in-run. Actual raw events and `s2-after-native-recreation` snapshots contain the evidence. The reviewer independently reconstructed GRANTED for current generation5, Slint surface22 and egui surface653, before a new-generation press. It is not an eighth recorded observation and does not increase the91 raw assertion count. Audit checks reject missing/mismatched supporting data.

## Remaining limits

Zero-delta and native repeat delivery remain UNVERIFIED. Unsolicited/background mapping and immediate private Core cache erasure are explicitly UNVERIFIED. NOT_GRANTED/TRANSITIONED classification has offline controls, not observed runtime examples in this fixture. A filtered getter is not treated as proof of erased private fields. No universal compositor focus guarantee or renderer superiority is claimed.

No edits, candidate execution or compositor execution were performed by the reviewer. [Evidence and exact diff](../results/2026-11-05-s2-user-open-focus/README.md).
