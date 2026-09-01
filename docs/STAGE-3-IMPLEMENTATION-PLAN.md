# Owlenshell Stage 3 Implementation Plan: Separate Hosts and Widgets

## 1. Current Stage 2 Baseline

- **Canonical Commit HEAD**: `5d98e9590427a43e34c0de7b6b38aeefa5d7ae84` (`feat: add session integration boundary`).
- **Test Suite**: 97 tests passing across 7 integration suites (`coordinator`, `domain`, `persistence`, `placement`, `platform_state`, `render_bridge`, `session`).
- **Live Status**: Manual Hyprland smoke and UWSM-context launch smoke (`uwsm app -- ./target/release/owlenshell`) verified passing.
- **Architectural Layering**: Stage 2 established clean ownership for `OutputManager` (canonical outputs), `RenderBridge` (Slint adapters by `HostId`), `SurfaceManager` (native surfaces and `SurfaceGeneration` tracking), `WaylandPlatform` (Wayland dispatch, connection, raw input), `Coordinator` (thin logical reconciliation, placement, drag, and persistence), and `Session` (standalone default with explicit UWSM boundary).

---

## 2. Stage 3 Objective

Stage 3 sits strictly ABOVE the proven Stage 2 platform stack. Its purpose is to decouple surface container policy (**Hosts**) from visual and functional content (**Widgets**).

### Required Stage 3 Outcomes
1. **Generic Host Policies**: Introduce data-driven generic `HostSpec` and `HostState` (e.g. `BarHost` policy vs `FloatingHost` policy) rather than branching on hard-coded host kinds.
2. **Generic Widget Contracts**: Introduce `WidgetSpec`, `WidgetState`, and widget content contracts (`ClockWidget`, `CalendarWidget`) decoupled from native surface lifecycle and layer-shell policies.
3. **Widget State Survival**: Widget content state (e.g., calendar month/year) survives host hide/show and native `SurfaceGeneration` destruction/recreation.
4. **Generic Command Model**: Replace calendar-specific toggle logic with generic typed host commands (`ToggleHost`, `ShowHost`, `HideHost`).
5. **Public Spec-Driven Default Shell**: Construct the default shell composition from public domain specs rather than private bootstrap branches.
6. **Elimination of PoC Coupling**: Fully remove `SurfaceKind`, `poc_compat.rs`, and widget-specific branching from runtime reconciliation and drag handling.

---

## 3. Current Coupling Inventory

The Stage 2 codebase contains residual PoC coupling that Stage 3 systematically eliminates:

| Residual Coupling | Current Location | Problem / Violation | Target Generic Replacement |
| :--- | :--- | :--- | :--- |
| `SurfaceKind` enum (`Bar`, `Calendar`) | `src/model.rs`, `src/runtime/coordinator.rs` | Couples runtime host state to hard-coded visual component names | Data-driven `HostSpec` and `HostMode` |
| `is_bar_host`, `is_calendar_host` | `src/runtime/poc_compat.rs` | Branches on surface kind for first-frame milestones and drag authorization | Generic host spec policies (e.g., milestone declaration, movement policy) |
| `surface_spec_for` | `src/runtime/poc_compat.rs` | Hard-codes layer, size, and exclusive zone policies per `SurfaceKind` | Direct conversion from `HostSpec` to `SurfaceSpec` |
| `compute_attach_geometry` | `src/runtime/poc_compat.rs` | Branches on `SurfaceKind` to compute bar vs calendar margins and sizes | Generic geometry computation derived from `HostSpec` and `HostPlacementConstraints` |
| `HANDLE_HEIGHT = 28.0` | `src/model.rs`, `src/runtime/poc_compat.rs` | Hard-codes calendar drag area at top of component | `HostMovementPolicy` / widget drag handle metadata |
| Hard-coded callbacks in `main.rs` | `src/main.rs` | `bar.on_open_calendar` and `cal.on_dismiss` directly manage adapter lifecycles | Generic host actions emitting `ToggleHost` / `HideHost` |
| `LogicalHostState.kind` | `src/runtime/coordinator.rs` | Stores `SurfaceKind` rather than generic `HostSpec` | `HostState` owning `HostSpec` and `attached_widgets: Vec<WidgetId>` |

---

## 4. Target Host / Widget Runtime Model

```
Shell Composition
 ├── Host (HostId)
 │    ├── HostSpec (Mode, Layer, Size, ExclusiveZone, Keyboard, MovementPolicy)
 │    ├── HostState (Logical Visibility, PlacementDecision, TopologyGeneration)
 │    ├── Attached Widgets (Vec<WidgetId>)
 │    └── Slint Root WindowAdapter (1 per Host, keyed by HostId)
 │
 └── Widget (WidgetId)
      ├── WidgetSpec (Title, EnabledByDefault)
      ├── WidgetState (Enabled, Content/Model State)
      └── Visual Content Binding (Clock, Calendar, etc.)
```

### Invariants
1. **Zero Native Knowledge in Widgets**: A `Widget` does not know `OutputId`, `SurfaceToken`, `SurfaceGeneration`, layer-shell margins, or exclusive zones.
2. **Zero Content Knowledge in Hosts**: A `Host` defines spatial, layer, and movement rules. It does not contain widget-specific business logic.
3. **Single Source of Truth for Containment**: `HostState.attached_widgets` is the canonical store of host-to-widget containment.
4. **Stable Widget State**: Widget data structures live in a widget state registry in the runtime and survive native surface teardown and recreation.

---

## 5. Slint Composition Strategy & Constraints

### Slint Constraints Analysis
- Pinned Slint version compiles static component hierarchies (`.slint` files into Rust structs).
- Dynamic runtime nesting of arbitrary compiled Slint component instances into a single generic Slint window root is not supported out-of-the-box without unified Slint root containers or multi-window architecture.
- In Owlenshell, **one Native Layer Surface = one Slint WindowAdapter = one Host**.

### Selected Strategy: Typed Host Root Windows with Generic Widget Controllers
- **Stage 3 Scope**: Maintain clean separation at the Rust logical and runtime layer:
  - `BarHost`: Owns `HostId("bar")`, `HostSpec::bar()`, and attaches `WidgetId("clock")`.
  - `FloatingHost`: Owns `HostId("calendar_popup")`, `HostSpec::floating()`, and attaches `WidgetId("calendar")`.
- **Slint Root Composition**: Each host binds to its Slint root component (`Bar` or `Calendar`) through `RenderBridge`.
- **Decoupled Controllers**: `ClockWidget` and `CalendarWidget` encapsulate the UI model manipulation (clock formatting, calendar week calculation, dismiss/toggle dispatch) without referencing `WaylandPlatform` or `SurfaceManager`.

---

## 6. Ownership & Responsibility Matrix

| Subsystem / Type | Primary Responsibility | Forbidden Knowledge |
| :--- | :--- | :--- |
| `HostSpec` | Immutable spatial & protocol rules (`HostMode`, `HostLayer`, `HostSize`, `ExclusiveZonePolicy`, `HostMovementPolicy`) | Widget rendering, UI state, Wayland protocol types |
| `HostState` | Mutable runtime logical state (`visible`, `attached_widgets`, `saved_placement`, `current_decision`, `decision_generation`) | Native SCTK handles, Slint window pointers |
| `WidgetSpec` | Generic metadata for widgets (`WidgetId`, `title`, `enabled_by_default`) | Placement engine, output geometry, display servers |
| `WidgetState` | Mutable logical state and content state of widget | Layer-shell anchors, surface tokens, margins |
| `Coordinator` | Orchestrates logical reconciliation, host command dispatch, drag tracking, and drop persistence | Widget UI markup, raw Wayland types |
| `RenderBridge` | Slint platform adapter management keyed strictly by `HostId` | Widget hierarchies, layer-shell placement |
| `SurfaceManager` | Native Wayland surface creation, sizing, and monotonic `SurfaceGeneration` tracking | Widget state, host movement policies |

---

## 7. Generic Host Command & Action Model

### Host Commands
Instead of calendar-specific toggle routines, runtime interaction is driven by generic typed commands:

```rust
pub enum HostCommand {
    Show(HostId),
    Hide(HostId),
    Toggle(HostId),
}
```

`Coordinator` executes these commands:
- `Show(host_id)`: Sets `host.visible = true`, requests reconcile/attach.
- `Hide(host_id)`: Sets `host.visible = false`, calls `surface_manager.retire(&host_id)`, cancels active drag targeting that host.
- `Toggle(host_id)`: Flips `host.visible` and executes corresponding show or hide transition.

### Widget Action Handling
Widgets declare interactive callbacks using generic closures or action emitters:
- `ClockWidget` on-click $\rightarrow$ emits `HostCommand::Toggle(target_host_id)`.
- `CalendarWidget` on-dismiss $\rightarrow$ emits `HostCommand::Hide(self_host_id)`.

---

## 8. Public Spec-Driven Default Shell

Default composition is expressed as pure, typed data in `src/domain/default_shell.rs` (or `src/runtime/default_shell.rs`):

```rust
pub struct ShellSpec {
    pub hosts: Vec<HostSpec>,
    pub widgets: Vec<WidgetSpec>,
    pub attachments: Vec<(HostId, WidgetId)>,
}

impl ShellSpec {
    pub fn default_shell() -> Self {
        Self {
            hosts: vec![
                HostSpec::default_top_bar(HostId::new("bar")),
                HostSpec::default_floating(HostId::new("calendar_host")),
            ],
            widgets: vec![
                WidgetSpec::new(WidgetId::new("clock"), "Clock"),
                WidgetSpec::new(WidgetId::new("calendar"), "Calendar"),
            ],
            attachments: vec![
                (HostId::new("bar"), WidgetId::new("clock")),
                (HostId::new("calendar_host"), WidgetId::new("calendar")),
            ],
        }
    }
}
```

---

## 9. Incremental Migration Sequencing

Stage 3 is divided into **6 small, reviewable steps**:

```
Step 1: Make Domain HostSpec & WidgetSpec Load-Bearing
Step 2: Generic HostState & Attachment in Coordinator
Step 3: Decouple ClockWidget & BarHost
Step 4: Decouple CalendarWidget & FloatingHost (Widget State Survival)
Step 5: Generic Host Commands & Public Default Shell Composition
Step 6: Remove poc_compat.rs, SurfaceKind, and Run Full Regression
```

---

### Step 1: Make Domain HostSpec & WidgetSpec Load-Bearing
- **Purpose**: Upgrade Stage 0 skeletal `HostSpec`, `HostState`, `WidgetSpec`, `WidgetState` to support full Stage 3 host policies without breaking Stage 2 runtime.
- **Files Touched**: `src/domain/host.rs`, `src/domain/widget.rs`, `src/platform/surface_manager.rs`, `tests/domain.rs`.
- **Changes**:
  - Add helper constructors on `HostSpec`: `HostSpec::default_top_bar(id, height)`, `HostSpec::default_floating(id)`.
  - Implement direct conversion `From<&HostSpec> for SurfaceSpec`.
  - Add conversion from `HostSpec` to `HostPlacementConstraints`.
- **Removed Code**: Redundant temporary conversions.
- **Tests**: Verify `HostSpec` converts accurately to `SurfaceSpec` and `HostPlacementConstraints` across all modes.
- **Gate**: 97 tests pass + new domain conversion tests. No runtime changes.

---

### Step 2: Generic HostState & Attachment in Coordinator
- **Purpose**: Refactor `Coordinator` internal storage from `LogicalHostState` (which used `SurfaceKind`) to `HostState` holding `HostSpec` and `attached_widgets: Vec<WidgetId>`.
- **Files Touched**: `src/runtime/coordinator.rs`, `src/runtime.rs`, `tests/coordinator.rs`.
- **Changes**:
  - `Coordinator` tracks `BTreeMap<HostId, HostState>`.
  - `Coordinator::attach_host` accepts `(HostSpec, Option<SavedPlacement>)`.
  - Sizing, geometry, and placement resolution derive parameters directly from `HostSpec` rather than `SurfaceKind`.
  - S2 drag handle height and first-frame milestones are derived from `HostSpec` policies (`HostMovementPolicy` / milestone flags).
- **Removed Code**: `LogicalHostState` struct.
- **Tests**: All Stage 2 placement, drag, fallback, and topology tests pass using generic host specs.
- **Gate**: All existing tests pass.

---

### Step 3: Decouple ClockWidget & BarHost
- **Purpose**: Extract clock formatting and tick logic into a dedicated `ClockWidget` controller.
- **Files Touched**: `src/runtime/widgets/clock.rs` (new), `src/runtime/widgets.rs` (new), `src/main.rs`.
- **Changes**:
  - `ClockWidget` encapsulates time formatting (`chrono::Local::now()`) and updates Slint `Bar` component.
  - Bar host interaction emits generic action.
- **Removed Code**: Inlined clock formatting and direct bar timer mutations in `main.rs`.
- **Tests**: Unit test for `ClockWidget` formatting and update trigger without requiring native surface.
- **Gate**: Automated test pass; clock updates reliably.

---

### Step 4: Decouple CalendarWidget & FloatingHost (Widget State Survival)
- **Purpose**: Extract `CalendarWidget` state and controller, ensuring calendar UI model state survives host hide/show and native recreation.
- **Files Touched**: `src/runtime/widgets/calendar.rs` (new), `src/runtime/widgets.rs`, `src/runtime/coordinator.rs`, `tests/coordinator.rs`.
- **Changes**:
  - `CalendarWidgetState` stores current displayed month/year and week model.
  - `CalendarWidget` controller manages Slint `Calendar` component binding.
  - When floating host is hidden (`surface_manager.retire`), `CalendarWidgetState` remains stored in runtime.
  - When floating host is re-shown or recreated on output change, `CalendarWidgetState` is reapplied to component without resetting to current date.
- **Removed Code**: Ad-hoc calendar state reconstruction in `main.rs`.
- **Tests**:
  - State survival test: Change calendar month to next year $\rightarrow$ hide host $\rightarrow$ show host $\rightarrow$ assert displayed month is preserved.
  - Recreation survival test: Change calendar month $\rightarrow$ trigger cross-output move $\rightarrow$ assert month preserved across new `SurfaceToken`.
- **Gate**: All tests pass including widget survival proofs.

---

### Step 5: Generic Host Commands & Public Default Shell Composition
- **Purpose**: Introduce `ShellSpec::default_shell()` and generic `HostCommand` dispatch (`Show`, `Hide`, `Toggle`).
- **Files Touched**: `src/domain/default_shell.rs` (new), `src/runtime/coordinator.rs`, `src/main.rs`, `tests/coordinator.rs`.
- **Changes**:
  - Add `Coordinator::execute_command(&self, cmd: HostCommand)`.
  - Wire `ClockWidget` to dispatch `HostCommand::Toggle(HostId::new("calendar_host"))`.
  - Wire `CalendarWidget` dismiss to dispatch `HostCommand::Hide(HostId::new("calendar_host"))`.
  - `main.rs` initializes shell from `ShellSpec::default_shell()`.
- **Removed Code**: Private calendar toggle closure and hard-coded show/hide branches in `main.rs`.
- **Tests**: Generic command execution tests for `Show`, `Hide`, `Toggle`.
- **Gate**: `main.rs` is a clean composition root.

---

### Step 6: Remove poc_compat.rs, SurfaceKind, and Run Full Regression
- **Purpose**: Complete Stage 3 cleanup by deleting all legacy PoC compatibility shims and verifying full system integrity.
- **Files Touched**:
  - Delete `src/runtime/poc_compat.rs`.
  - Delete `SurfaceKind` from `src/model.rs`.
  - Clean `src/runtime.rs` exports.
  - Update any test helper still referencing old types.
- **Removed Code**: `src/runtime/poc_compat.rs`, `SurfaceKind`, `is_bar_host`, `is_calendar_host`, `surface_spec_for`, `compute_attach_geometry`.
- **Tests**: Full suite regression across all test binaries.
- **Gate**: Zero compiler warnings, 0 obsolete symbols, all tests green, live Hyprland smoke pass.

---

## 10. Deletion Checklist

- [ ] `src/runtime/poc_compat.rs` (deleted)
- [ ] `SurfaceKind` enum in `src/model.rs` (deleted)
- [ ] `is_bar_host()` and `is_calendar_host()` (deleted)
- [ ] `surface_spec_for()` (deleted, replaced by `SurfaceSpec::from(&HostSpec)`)
- [ ] `compute_attach_geometry()` (deleted, replaced by generic host geometry calculation)
- [ ] `bar_output_id()` (deleted)
- [ ] Ad-hoc calendar toggle closures in `src/main.rs` (deleted, replaced by generic host commands)

---

## 11. Technical Risk & Mitigation Analysis

| Identified Risk | Severity | Mitigation Strategy |
| :--- | :--- | :--- |
| **Slint Component Lifetime on Host Hide** | Medium | When a host is hidden, native surface is retired, but Slint component adapter is retained in `RenderBridge`. Widget state is tracked in `CalendarWidgetState` so re-attaching reapplies state cleanly. |
| **First-Frame Milestone Tracking** | Low | Instead of checking `is_bar_host`, `HostSpec` declares `primary_milestone: bool` (set on the main bar host) so `shell-ready` milestone fires generically. |
| **Drag Authorization on Generic Hosts** | Low | Instead of checking `is_calendar_host`, `Coordinator` checks `host_spec.movement_policy == HostMovementPolicy::Movable`. |
| **Premature Generalization** | Low | Keep widget types concrete (`ClockWidget`, `CalendarWidget`) while making the container and dispatch generic. |

---

## 12. Final Stage 3 Acceptance Gates

1. **Rust Verification**:
   - `cargo fmt --all -- --check` passes.
   - `cargo clippy --all-targets --all-features -- -D warnings` passes.
   - `cargo test --all-targets` passes (all existing + new Stage 3 tests).
   - `cargo build --release` compiles cleanly.
   - `git diff --check` passes.
2. **Structural Invariants**:
   - Zero occurrences of `SurfaceKind` or `poc_compat`.
   - `HostSpec` and `WidgetSpec` are the sole logical containers.
   - `CalendarWidgetState` survives hide/show and cross-output recreation.
   - `main.rs` bootstraps solely from `ShellSpec::default_shell()`.
3. **Live Hyprland Smoke**:
   - Bar clock and calendar open/dismiss/drag/drop operate identically to Stage 2 baseline.
