# S2 User-Open Focus adoption

> **Curated publication copy.** Source: `6304979719f1388e6ff98d549224107d0d45f284`, `docs/bakeoffs/renderer/decisions/2026-11-05-s2-user-open-focus-adoption.md`. Private paths/identifiers are redacted and links to omitted material lead to the [excluded-source catalogue](../SOURCE-REFERENCES.md). Original source and hashes are listed in [provenance](../provenance.json). This is a dated record: historical ADR statuses, approvals and test counts describe their original scope, not new acceptance. Current initial selection is [Accepted ADR-0006](../../../ADR/0006-renderer-selection.md); Gate P remains NO-GO. References to the repository, its history, implementations or raw evidence describe the retained original experimental repository, not this curated branch. Commands are historical descriptions, not runnable reproduction instructions or execution authorization for this curated snapshot.

Status: **Accepted by the maintainer**, 2026-11-05 Europe/Berlin. Evidence/source base: signed `9d73831b915445895f0eec61db5d3267a74bfa30`.

Contract ID: `USER-OPEN-FOCUS-S2-r1`.

An explicit user activation of the clock may map the calendar with OnDemand keyboard interactivity. The compositor is permitted to focus that calendar on mapping, before a calendar click. This is OwlenShell's user-open UX contract; it does not require all compositors to grant focus on mapping.

The previous requirement that the probe retain focus after a user clock activation is prospectively superseded for this S2 case. The historical requirement, assertion and original FAIL/NO-GO evidence remain unchanged in their signed revisions and evidence bundles. No historical result becomes PASS through this decision.

Actual native keyboard events, current surface generation, Core focus and UI delivery must agree. Granted focus requires calendar key delivery; no grant requires the prior probe's key delivery, recorded separately from routing failure. A focus change during the routing probe is insufficient attribution. If keyboard focus is unavailable after the preserved non-close click, keyboard-dependent controls are BLOCKED/UNVERIFIED with nonzero exit and cleanup, rather than spoofed focus or input-routing FAIL. Contradictory receipts or wrong/duplicate delivery remain FAIL.

All unrelated S2 expectations remain: bar None, one clock activation/one intended Toggle(calendar), actual Coordinator state, typed keys/modifiers, exactly-once actions, scrolling, cancellation, stale-generation rejection, teardown ordering and cleanup. The approved runs remain one output, scale1, isolated Nested-Niri, using the existing runtime binary. Core ownership and renderer policy are unchanged.

Unsolicited/background Show and focus behavior on recreation of an already-unfocused calendar remain UNVERIFIED. The existing focused-calendar recreation control is not evidence for those cases. A filtered Core focus getter alone does not prove immediate erasure of private retired-token/key-cache fields; active native/UI teardown and generation rejection have separate evidence.

This authorization covers the narrow assertion correction, focused tests and private Slint/egui S2 runs with independent review. It does not authorize S3–S6, production integration, comparative measurements, renderer selection or enabling the private Niri diagnostic. ADR-0006 remains Proposed and all historical NO-GOs remain preserved.

[Triage and alternatives](../SOURCE-REFERENCES.md#source-e432666ee58f21bb). [New validation report](../results/2026-11-05-s2-user-open-focus/README.md).
