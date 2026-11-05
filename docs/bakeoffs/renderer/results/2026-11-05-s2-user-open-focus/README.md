# S2 User-Open Focus validation

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/results/2026-11-05-s2-user-open-focus/README.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Contract: `USER-OPEN-FOCUS-S2-r1`, [accepted by the maintainer](../../decisions/2026-11-05-s2-user-open-focus-adoption.md) from signed base `9d73831b915445895f0eec61db5d3267a74bfa30`.

This new evidence series validates the adopted user-clock-open contract. It does not reinterpret the original no-focus-on-map failure or its historical NO-GO. Original [S2 evidence](../../SOURCE-REFERENCES.md#source-dc2f658b56f18a04), [triage](../../SOURCE-REFERENCES.md#source-e432666ee58f21bb), reviews and signed revisions remain unchanged.

## Scope and assertion change

A clock activation may map the Top/OnDemand calendar; actual compositor focus may arrive before another calendar click. Focus-on-map is permitted, not required universally. The previous assertion that the probe must retain focus is removed prospectively, with the original preserved in Git/evidence.

The replacement correlates the actual current calendar native generation and surface ID with native keyboard Enter, token-scoped Core dispatch, UI receipt and Core/Coordinator state. It observes focus before and after an `a` probe. GRANTED requires the exact calendar press/release pair and no probe delivery. NOT_GRANTED requires the exact prior probe pair and no calendar key delivery; this is distinct from routing FAIL. A focus transition during that probe is insufficient attribution and blocks dependent coverage. After the retained non-close click, unavailable focus blocks keyboard controls as UNVERIFIED with cleanup/nonzero exit. Wrong surface/generation, contradictory receipts and duplicate/wrong-target keys remain FAIL.

The source diff also checks the exact intended Toggle(calendar) for each clock activation; records focus after reopen and generation replacement; and verifies dismissal removes the active native/UI registration, cancels held UI state and drops GPU resources before native destruction. Other existing pointer/key/scroll/focus-loss/restoration/recreation expectations remain. Focus events are neither filtered out nor forged.

The renderer executable, Core/SCTK ownership and presentation/input implementations are reused unchanged. The existing reviewed private harness, fixed input fixture and installed unpatched Niri are retained. There is no new runtime instrumentation, input policy mechanism, scheduler, performance sampler or generalized framework.

## Interpretation limits

Unsolicited/background mapping is UNVERIFIED and not exercised. The generation replacement control starts with a focused calendar; it does not establish behavior on recreation of an unfocused calendar. A filtered Core focus getter establishes absence of current focus; it does not expose immediate erasure of private retired-token/key-cache fields. Actual UI cancellation, native teardown, subsequent routing and generation rejection have separate evidence; private-cache erasure remains UNVERIFIED rather than being inferred from None.

Native autorepeat delivery and zero-delta delivery retain their existing UNVERIFIED status. Physical Wheel/Finger sources, generic keyboard/IME and secondary-button parity, visual highlight parity, scaling/multi-output and device recovery are not upgraded. No universal cross-compositor focus behavior, Gate R eligibility, performance, scene equivalence or renderer superiority follows.

## Results under the adopted contract

| Candidate | PASS | FAIL | UNVERIFIED | Client / harness exit |
|---|---:|---:|---:|---|
| Slint | 87 | 0 | 4 | 0 / 0 |
| egui | 87 | 0 | 4 | 0 / 0 |

Both private runs completed using signed harness `588ed633006337d62e76cdc44ba32d60188dc0f2` and the unchanged reviewed executable. Each accepted exactly8 lifecycle actions, alternating four Toggle(calendar) with four Hide(calendar), with6 native lifetimes (bar1, calendar5) destroyed. Each of the72 historical non-creation-focus assertions retains its previous PASS/UNVERIFIED status. The removed historical FAIL is absent only in this new prospective suite; the old dataset still contains it.

Each run records seven in-run focus observations, all GRANTED on this pinned Niri fixture. Native surface ID, current generation, Core dispatch, UI receipt and snapshot agree. The real `a` pair reaches the calendar rather than the prior probe. Keyboard Return/Space/Escape dismissal, probe restoration, close/reopen, held-button retirement/replacement and ordered active native/UI cleanup are verified for the recorded sequences. No bar keyboard focus is observed. A process-identity/group scan finds none of100 recorded owned identities/groups remaining; both cleanup receipts contain no errors.

There is an explicit instrumentation gap: a planned dedicated pre-click generation-replacement focus observation was omitted from the in-run list. Its actual pre-click native focus events and Coordinator snapshot were nevertheless captured. The [separately labeled post-run reconstruction](../../SOURCE-REFERENCES.md#source-5bf5501533e775bc) records the snapshot hash, exact decompressed log prefix endpoint/hash, current generation5, underlying surface (Slint22, egui653), native Enter/Core/UI agreement and absence of a new-generation press before that endpoint. The reviewer independently reproduced this. It is **not** counted as an eighth in-run capture or another raw PASS assertion. Missing/mismatched underlying evidence makes the audit fail.

The [offline audit](../../SOURCE-REFERENCES.md#source-85ff64570b8f1042) verifies33 matching canonical domain/placement/focus snapshots, exact action identities, native protocol destruction, cancellation/resource ordering and preservation of all unrelated assertion outcomes. This is functional state agreement, not pixel/workload/performance equivalence. The adopted contract and attempted functional controls pass at this scope; unverified cases remain open, so this is not complete production acceptance or an authorization to start S3.

The four raw UNVERIFIED checks are zero-delta delivery, native autorepeat delivery, unsolicited/background mapping and immediate private Core cache erasure. The NOT_GRANTED/TRANSITIONED classification paths have offline regression evidence but were not encountered at runtime here; no universal grant or cross-compositor behavior is inferred.

## Review, failures and reproduction

[Independent review](../../reviews/2026-11-05-s2-user-open-focus.md) covers adoption, exact assertion diff, isolation, metadata, evidence and preservation. Pre-runtime review found two MEDIUM issues: explicit egui UI generation was not checked, and correlated Enter→Leave during a key probe could be mislabeled FAIL. Both were corrected before execution. A regression then caught duplicate native-created boundary records being accepted through last-record selection; exactly one boundary is now required. Final19 offline harness tests pass; five evidence-audit controls reject empty/missing states, a stale saved receipt and missing native focus. No new Rust build/test claim is made: runtime Rust/Slint/manifests/lock inputs are unchanged and54 source input hashes are verified against the prior build evidence.

[Exact assertion/contract diff](../../SOURCE-REFERENCES.md#source-03c6384bf845d72e), [commands](../../SOURCE-REFERENCES.md#source-f4bef1421d0ee38c), [source/binary provenance](../../SOURCE-REFERENCES.md#source-e928f362b88819bb), [raw runs](../../SOURCE-REFERENCES.md#source-4982b31b8459fa4c), [historical preservation hashes](../../SOURCE-REFERENCES.md#source-7a89966d066057be), [cleanup audit](../../SOURCE-REFERENCES.md#source-70efd73811944ca7) and [test logs](../../SOURCE-REFERENCES.md#source-60a71dcd0f2a2ef8) permit reproduction. Use a separate checkout of the final evidence commit; the audit verifies its harness-source binding before import. The executable was reused, not rebuilt from this documentation revision. To execute again, use fresh private paths and the existing isolated harness within an authorized scope; the evidence reader itself launches no candidate. Raw logs may be losslessly gzip-compressed with original byte lengths/hashes recorded. Generated renderer caches, binaries and nonregular runtime objects are excluded from Git; isolated temporary originals are retained.

The original337 S2 files and six focus-triage artifacts remain byte-identical to signed base `9d73831b915445895f0eec61db5d3267a74bfa30`. This adoption prospectively corrects the clock-open requirement; it never promotes the historical FAIL/NO-GO or exploratory measurements into new acceptance evidence.
 No production integration or renderer selection is authorized. ADR-0006 remains Proposed; the private Niri diagnostic remains disabled. No S3–S6 work is authorized by this contract adoption.
