# Current consolidation guide and B0–B6 alignment

**Current planning-status entry point, 2026-11-05.** [MDR-0001](MAINTAINER-DECISIONS.md) records D01–D11 approved in principle and P01–P11 approved policy. [B0 contracts](B0-ARCHITECTURE-CONTRACTS.md) operationalize that direction as proposed specifications; [gaps](SPECIFICATION-GAPS.md) retain unselected engineering details. [New independent review](B0-CONSOLIDATION-REVIEW.md), [new provenance](B0-CONSOLIDATION-SOURCES.json).

This is an additive consolidation. All original nine drafts remain byte-identical historical inputs; their old open/proposed wording must now be read with MDR-0001's status overlay. No accepted ADR, source, dependency, configuration, native surface, service or benchmark is changed. This guide approves no implementation or B0 completion.

## Approval levels

| Item | Current status | What it does not mean |
|---|---|---|
| D01–D11 | MAINTAINER APPROVED IN PRINCIPLE | Unspecified mechanisms, public schema, provider defaults and exact environments are not selected automatically |
| P01–P11 | MAINTAINER APPROVED POLICY | Current code does not become compliant merely because a policy was selected |
| K01–K11 | PROPOSED B0 CONTRACT SPECIFICATIONS | Independent review is not maintainer acceptance or implementation authorization |
| G01–G15 | OPEN SPECIFICATION GAPS | They cannot silently change approved policy; resolve before each named prerequisite |
| C01–C19 | OPEN FINDINGS / FOLLOW-UP | Policy clarification or target selection does not close code, documentation or acceptance work |
| B0 | CONTRACT PLANNING AUTHORIZED; completion not declared | B1–B6 code/runtime work is not authorized |
| Gate P / historical runtime evidence | Gate P UNPASSED; historical S1–S6 scope unchanged | No broad support claim, exploratory promotion or Niri multi-output closure |

## Proposed revisions to the preserved nine drafts

These are explicit proposed revisions/read-through instructions, not applied edits. This meets consolidation needs while preserving the audit record. A later authorized documentation revision may apply the status banners/deltas without rewriting findings or accepted ADRs.

| Original input | Proposed current-status revision / interpretation |
|---|---|
| [README.md](README.md) | Add a current-guide/MDR notice; retain original baseline, findings, runtime limits and test-isolation history |
| [OPEN-DECISIONS.md](OPEN-DECISIONS.md) | Mark D01–D11 approved in principle via MDR; P01–P11 are selected policy. Replace only their old decision-status presentation; unresolved mechanisms become G gaps, not reopened policy questions |
| [BASE-ARCHITECTURE-V1-PROPOSAL.md](BASE-ARCHITECTURE-V1-PROPOSAL.md) | Retain ownership proposal; read format as versioned TOML, precedence as explicit > permitted saved > defaults, writer as single, valid apply/save-failure outcomes separately, no auto migration/reconnect. Exact interfaces remain K/G specifications, not code |
| [MILESTONES-AND-GATES.md](MILESTONES-AND-GATES.md) | Use aligned B0–B6 table below. Remove the implication that all D policy choices remain unapproved; keep implementation/validation authorization and all original acceptance cases |
| [IMPLEMENTATION-EVIDENCE.md](IMPLEMENTATION-EVIDENCE.md) | Keep source/test findings unchanged. MDR adds requirements only; it supplies no new PASS or source integration. No tests rerun in this consolidation |
| [CONFLICT-REGISTER.md](CONFLICT-REGISTER.md) | Preserve all 19 IDs, evidence, severity and findings. Add status references to the overlay below; never label resolved solely from approval |
| [GPUI-LINUX-REFERENCE.md](GPUI-LINUX-REFERENCE.md) | Preserve pinned source assessment, conceptual recommendations and licensing constraint. D11 permits independent concepts, not code copying or an upstream executor choice |
| [REVIEW.md](REVIEW.md) | Preserve the prior nine-draft review and its limits. The new review covers only this consolidation; no retroactive extension of the old verdict |
| [SOURCES.json](SOURCES.json) | Preserve original provenance/hashes. New provenance records its byte identity and new files separately; never re-hash history to imply new approved policies were in the original snapshot |

## Conflict status overlay — no findings closed

Architectural direction may now be selected while implementation and evidence remain missing. Every status below is **OPEN**; documentation-only conflicts may need reviewed documentation follow-up rather than code. The [original evidence/severity](CONFLICT-REGISTER.md) is authoritative for what was found.

| Conflict | Status | Approved planning basis | Remaining follow-up |
|---|---|---|---|
| C01 | OPEN | D02/D03; K04 | B1–B3; generation guards remain missing in canonical paths |
| C02 | OPEN | D07/P09; K05 | B1/B2; registry-invalidation/drop ordering remains unchanged |
| C03 | OPEN | D02; K01/K02 | B1; concrete Slint coupling remains in current application |
| C04 | OPEN | D03; K02/K03 | B0/B1; terminology/policy direction clarified, original documents not revised |
| C05 | OPEN | D03/P03; K03 | B1/B4; canonical registry consolidation not implemented |
| C06 | OPEN | D06/P04/P05; K08 | B4; TOML selected, field/schema specification and parser not implemented |
| C07 | OPEN | D09; K11 | Session work deferred; no-op/unsupported capabilities remain actual code |
| C08 | OPEN | D05/P10; K01/K10 | B1/B5 as scoped; blocking enrichment remains in current code |
| C09 | OPEN | D04/P02; K07 | B2; three observations separated by policy, trigger/timeout still G02 |
| C10 | OPEN | D07/P09; K11 | B2; full application shutdown/loss handling not implemented |
| C11 | OPEN | D04/P11; K06 | B1 minimum/B3; environment-specific input/quality tests remain required |
| C12 | OPEN | D02/D06; K02/K08 | B1/B4; single geometry authority still requires specification/code |
| C13 | OPEN | D06/D08; P04–P08; K08/K09 | B4; precedence/writer/error/migration policies selected, mechanisms and proof absent |
| C14 | OPEN | D03/P03; K03 | B1/B4; attachment/controller validation and materialization unchanged |
| C15 | OPEN | D01/P01/P11; K03/K06 | B1 one-output scope does not close B3 multi-output or Niri BLOCKED |
| C16 | OPEN | D02/D05/P10; K01/K07/K10 | B2/B5; wake/deadline/service integration not implemented |
| C17 | OPEN | D01/P01 | B0/B6; Base/Product distinction selected, Product v1 obligations unchanged |
| C18 | OPEN | MDR status overlay | Documentation follow-up; historical findings/statuses remain preserved |
| C19 | OPEN | K09 and retained private-state validation rule | Tests still reach default-state path; no test-source isolation fix made |

## Aligned milestone progression

Original test cases, private-environment safeguards, rollback/stop conditions and distinctions between native-wire/protocol/visual evidence carry forward. The table below adds policy alignment, not permission to run or a reduction of the original acceptance contract.

| Milestone | Approved direction / proposed deliverable | Dependencies and specifications | Acceptance / rollback boundary |
|---|---|---|---|
| B0 — contracts and invariants | MDR status record, K01–K11, bounded gap ownership and reviewed B1 brief | Maintainer review of K package; G14 and minimum G03/G04 for implementation-ready exit | Contract planning only. No code, completion claim, migration or automatic B1 approval |
| B1 — ownership boundary | Concrete Slint/Core seam, native/UI owner thread, canonical logical state and generation-safe effects; one declared compositor/renderer, one output, scale 1 | B0 exit and separate authorization; G03/G04/G14; G01 and minimum G02/G05 before runtime validation | Existing bar/clock/calendar parity, old-token negatives, one presenter and partial-init rollback. Retain known-good composition; private state only; no new Linux provider or generic renderer framework |
| B2 — scheduling, lifecycle and shutdown | Dirty/deadline/native-admission integration, separate readiness observations, ordered shutdown and complete Wayland-loss stop | B1; G02/G13, extend G04; respect protected repository/writer state even before B4 | Fake-clock + separately authorized private signal/failure controls. Lost connection cannot prove server destruction/flush; no reconnect/automatic restart; forced cleanup is not graceful PASS |
| B3 — output, input and scaling | Per-output context/state, replacement, declared input/region/focus and scale compatibility | B2; expanded G03/G05 and approved genuine-output environment | Preserve original five-scale compatibility contract; native fractional and broad visual qualification stay separate. SwayFX remains compositor-specific, Niri multi-output BLOCKED unchanged; unavailable capabilities remain unverified/blocked |
| B4 — configuration and persistence | Versioned TOML, shared validator, explicit > permitted saved > defaults, single writer, truthful apply/save outcomes, protected state and no auto migration | B1 + B2; G06–G11 | Tests on private files and fault stages. Last-valid state protected; valid in-memory changes may remain unsaved with diagnostics; unknown future schema not overwritten, including after fallback/shutdown. Failed configuration/application does not replace valid active revision |
| B5 — minimal optional services | Fake provider then one approved narrow non-owning integration; bounded messages, typed outcomes and actual cancellation/completion | B2 + B4; G12/G13/G15; no premature executor/default choice | Shared consumers, saturation, stale-run/reconnect and stop-completion controls. Significant commands/errors not silently lost; provider failure remains explicit. No default notification/tray ownership or production service mutation |
| B6 — qualification and portability evidence | Evidence for the declared supported base and required Gate P obligations | Applicable B3/B4/B5 results plus approved target matrix and remaining acceptance criteria | Portability targets are not certifications. Keep Product v1/Gate P requirements, physical/hardware/quality gaps and separate benchmark authorization. No rollout/integration is authorized here |

The dependency graph is **B0 → B1 → B2 → B3**, **B1 + B2 → B4**, **B2 + B4 → B5**, then **applicable B3/B4/B5 evidence → B6**. B4 need not wait for B3. A policy assigned primarily to B4 still applies to any earlier real-state write; earlier bounded tests must use exclusively owned private state. General IPC and GUI are not prerequisites for B4's common validation path or P06's writer contract.

## Proposed ADR candidates — none adopted

These are titles/scopes for future review, not accepted ADR files or reservations of ADR numbers. Existing ADR-0006 and native/placement/compositor decisions remain unchanged. Merely enforcing an existing invariant does not require rewriting its ADR.

| Candidate | Proposed title and enduring scope | Dependency / timing |
|---|---|---|
| A01 | Owner-thread application and concrete Slint/native boundary — native authority, neutral boundary, one presenter and effect admission | D02; K01/K02/K04/K05; specify G03/G04/G14 before stabilizing the B1 contract |
| A02 | Canonical host/widget composition and lifetime identity — one controller/attachment truth, rehosting retention and UI/native incarnation separation | D03/P03; K03/K04; depends on A01's boundary, before public composition semantics freeze |
| A03 | Ordered shutdown and complete Wayland-loss stop — partial startup, completion, unsaved outcomes and no reconnect/automatic restart | D07/P07/P09; K05/K11; G13 before B2 implementation; automatic supervision would be a separate later decision |
| A04 | Versioned TOML configuration and persistence authority — precedence, permitted saved fill, single writer, publication/durability outcomes and protected future schema | D06/D08; P04–P08; K08/K09; G06–G11 before B4 compatibility and storage promises |
| A05 | Optional service execution, backpressure and completion — concrete demand/mode/worker/connection contract without generic framework | D05/P10; K10; only when G12/G15 justify a concrete B5 model, not a premature executor decision |
| A06 | Session/process integration authority — truthful capabilities and authoritative manager detection/launch/notification | D09; deferred until real session integration is needed; must not duplicate service supervision or make UWSM mandatory |
| A07 | Local IPC namespace and command authority — transport/admission/size/schema/path ownership and safe cleanup | D10; deferred until general IPC is separately requested. P06 enforcement alone does not adopt IPC |

P01/P02/P11 environment/readiness/quality criteria can be recorded in the scoped acceptance contract unless a new enduring product policy requires its own ADR. D11 requires licensing/permission/attribution records before any direct reuse, not a new architecture ADR merely to study concepts. No candidate above is an added maintainer approval.

## B0 handoff and blockers

There is no identified contradiction among the approved D/P policies that blocks review of the B0 contract package. **B0 is not declared complete or accepted by this consolidation.** A maintainer must review K01–K11 and a bounded implementation-ready B1 brief; G14 and the relevant token/retirement specifications G03/G04 remain pre-exit engineering work. Exact G01 environment selection and the B1 portions of G02/G05 gate runtime validation, not an implicit choice made here.

Later gap deadlines are explicit rather than all blocking the first seam. No policy approval closes C01–C19, authorizes B1–B6 implementation or turns scoped S1–S6 results into production acceptance. All files remain unstaged/uncommitted and no publication action is part of this work.
