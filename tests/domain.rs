use owlenshell::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostMovementPolicy, HostSize, HostSpec, HostState,
    KeyboardInteractivityPolicy,
};
use owlenshell::domain::ids::{HostId, OutputId, SeatId, WidgetId};
use owlenshell::domain::input::{can_move, MovementCapability};
use owlenshell::domain::output::{OutputIdentity, OutputPreference, OutputSnapshot};
use owlenshell::domain::placement::{
    Anchor, DeferredSurface, PlacedSurface, PlacementDecision, ResolveReason, SavedPlacement,
};
use owlenshell::domain::widget::{WidgetKind, WidgetSpec, WidgetState};
use owlenshell::platform::SurfaceSpec;

#[test]
fn typed_ids_are_distinct_and_well_formed() {
    let host_id = HostId::new("bar");
    assert_eq!(host_id.as_str(), "bar");
    assert_eq!(format!("{host_id}"), "bar");

    let widget_id = WidgetId::new("clock");
    assert_eq!(widget_id.as_str(), "clock");
    assert_eq!(format!("{widget_id}"), "clock");

    let seat_id = SeatId::new("seat-0");
    assert_eq!(seat_id.as_str(), "seat-0");
    assert_eq!(format!("{seat_id}"), "seat-0");

    let out_id = OutputId::new(42);
    assert_eq!(out_id.raw(), 42);
    assert_eq!(format!("{out_id}"), "output#42");
}

#[test]
fn movement_capability_and_host_policy_separation() {
    assert!(MovementCapability::GlobalPointerStream.is_draggable());
    assert!(MovementCapability::StableRelativeMotion.is_draggable());
    assert!(MovementCapability::CompositorMoveSession.is_draggable());
    assert!(!MovementCapability::Unavailable.is_draggable());

    assert!(can_move(
        HostMovementPolicy::Movable,
        MovementCapability::GlobalPointerStream
    ));
    assert!(!can_move(
        HostMovementPolicy::Movable,
        MovementCapability::Unavailable
    ));
    assert!(!can_move(
        HostMovementPolicy::Fixed,
        MovementCapability::GlobalPointerStream
    ));
}

#[test]
fn host_spec_is_generic_without_widget_variants() {
    let spec = HostSpec {
        id: HostId::new("floating-container"),
        namespace: "owlenshell-floating".into(),
        mode: HostMode::Float,
        layer: HostLayer::Top,
        size: HostSize::ContentPreferred,
        default_anchor: Anchor::TopRight,
        default_offset: (24, 40),
        exclusive_zone: ExclusiveZonePolicy::None,
        keyboard_interactivity: KeyboardInteractivityPolicy::OnDemand,
        movement_policy: HostMovementPolicy::Movable,
        primary_milestone: false,
        exit_on_close: false,
    };

    assert_eq!(spec.mode, HostMode::Float);
    assert_eq!(spec.layer, HostLayer::Top);
    assert_eq!(spec.movement_policy, HostMovementPolicy::Movable);

    let mut state = HostState::new(spec);
    assert!(state.visible);
    state.attach_widget(WidgetId::new("generic-widget-1"));
    assert_eq!(state.attached_widgets().len(), 1);
    assert_eq!(state.attached_widgets()[0].as_str(), "generic-widget-1");

    assert!(state.detach_widget(&WidgetId::new("generic-widget-1")));
    assert!(state.attached_widgets().is_empty());
}

#[test]
fn bar_host_policy_without_clock_knowledge() {
    let bar_spec = HostSpec::top_bar(HostId::new("bar_container"), 32);
    assert_eq!(bar_spec.id.as_str(), "bar_container");
    assert_eq!(bar_spec.mode, HostMode::Cover);
    assert_eq!(bar_spec.layer, HostLayer::Top);
    assert_eq!(bar_spec.exclusive_zone, ExclusiveZonePolicy::Fixed(32));
    assert_eq!(bar_spec.movement_policy, HostMovementPolicy::Fixed);
    assert!(bar_spec.primary_milestone);

    // Converts to platform SurfaceSpec accurately
    let surface_spec = SurfaceSpec::from(&bar_spec);
    assert_eq!(surface_spec.namespace, "owlenshell-bar");
    assert_eq!(surface_spec.mode, HostMode::Cover);
    assert_eq!(surface_spec.exclusive_zone, ExclusiveZonePolicy::Fixed(32));

    // Can host an arbitrary non-clock widget
    let mut state = HostState::new(bar_spec);
    state.attach_widget(WidgetId::new("system_monitor"));
    assert_eq!(state.attached_widgets()[0].as_str(), "system_monitor");
}

#[test]
fn floating_host_policy_without_calendar_knowledge() {
    let float_spec = HostSpec::floating(HostId::new("popup_container"));
    assert_eq!(float_spec.id.as_str(), "popup_container");
    assert_eq!(float_spec.mode, HostMode::Float);
    assert_eq!(float_spec.layer, HostLayer::Top);
    assert_eq!(float_spec.exclusive_zone, ExclusiveZonePolicy::None);
    assert_eq!(float_spec.movement_policy, HostMovementPolicy::Movable);
    assert!(!float_spec.primary_milestone);

    // Converts to platform SurfaceSpec accurately
    let surface_spec = SurfaceSpec::from(&float_spec);
    assert_eq!(surface_spec.namespace, "owlenshell-floating");
    assert_eq!(surface_spec.mode, HostMode::Float);
    assert_eq!(surface_spec.exclusive_zone, ExclusiveZonePolicy::None);

    // Can host an arbitrary non-calendar widget
    let mut state = HostState::new(float_spec);
    state.attach_widget(WidgetId::new("quick_notes"));
    assert_eq!(state.attached_widgets()[0].as_str(), "quick_notes");
}

#[test]
fn clock_and_calendar_widgets_contain_no_host_or_native_policy() {
    let clock = WidgetSpec::clock(WidgetId::new("clock_widget"));
    assert_eq!(clock.kind, WidgetKind::Clock);
    assert_eq!(clock.title, "Clock");
    assert!(clock.enabled_by_default);

    let calendar = WidgetSpec::calendar(WidgetId::new("calendar_widget"));
    assert_eq!(calendar.kind, WidgetKind::Calendar);
    assert_eq!(calendar.title, "Calendar");
    assert!(calendar.enabled_by_default);

    // Widget state exists independently of any Host or native surface
    let mut clock_state = WidgetState::new(clock.id);
    assert!(clock_state.enabled);
    clock_state.enabled = false;
    assert!(!clock_state.enabled);

    let cal_state = WidgetState::new(calendar.id);
    assert!(cal_state.enabled);
}

#[test]
fn widget_spec_and_state_are_decoupled_from_host_policy() {
    let widget_spec = WidgetSpec::new(
        WidgetId::new("weather-widget"),
        WidgetKind::Custom,
        "Weather",
    );
    assert_eq!(widget_spec.id.as_str(), "weather-widget");
    assert_eq!(widget_spec.kind, WidgetKind::Custom);

    let widget_state = WidgetState::new(widget_spec.id.clone());
    assert!(widget_state.enabled);
    assert_eq!(widget_state.id.as_str(), "weather-widget");
}

#[test]
fn saved_and_resolved_placement_are_distinct_types() {
    let saved = SavedPlacement {
        output: OutputPreference::new().with_name("DP-1"),
        anchor: Anchor::TopLeft,
        offset: (10, 20),
    };

    let resolved = PlacementDecision::Placed(PlacedSurface {
        target_output_id: OutputId::new(1),
        anchor: Anchor::TopLeft,
        offset: (10, 20),
        margins: (20, 0, 0, 10),
        reason: ResolveReason::ExactName,
        is_fallback: false,
        is_reachable: true,
    });

    let deferred = PlacementDecision::Deferred(DeferredSurface {
        anchor: Anchor::TopLeft,
        offset: (10, 20),
        margins: (20, 0, 0, 10),
        reason: ResolveReason::Deferred,
    });

    assert_ne!(saved.offset, (0, 0));
    assert_eq!(resolved.target_output_id(), Some(OutputId::new(1)));
    assert!(!resolved.is_fallback());
    assert!(resolved.is_reachable());
    assert!(!resolved.is_deferred());

    assert_eq!(deferred.target_output_id(), None);
    assert!(deferred.is_deferred());
    assert!(!deferred.is_reachable());
}

#[test]
fn output_identity_and_role_matching_evidence() {
    let ident1 = OutputIdentity {
        make: Some("Dell".into()),
        model: Some("U2720Q".into()),
        physical_size_mm: Some((600, 340)),
        serial: Some("SN123".into()),
    };

    let snapshot = OutputSnapshot {
        id: OutputId::new(1),
        name: Some("DP-1".into()),
        identity: ident1,
        logical_position: Some((0, 0)),
        logical_size: Some((3840, 2160)),
        logical_role: Some("primary".into()),
        scale: 2,
        preferred: true,
    };

    let pref = snapshot.to_preference();
    assert_eq!(pref.name.as_deref(), Some("DP-1"));
    assert_eq!(pref.serial(), Some("SN123"));
    assert_eq!(pref.make(), Some("Dell"));
    assert_eq!(pref.model(), Some("U2720Q"));
    assert_eq!(pref.physical_size_mm(), Some((600, 340)));
    assert_eq!(pref.logical_role.as_deref(), Some("primary"));
}

#[test]
fn host_spec_to_placement_constraints_conversion() {
    use owlenshell::domain::host::HostSize;
    use owlenshell::placement::HostPlacementConstraints;

    // 1. top_bar HostSpec preserves configured height and does not invent static width
    let bar_spec = HostSpec::top_bar(HostId::new("bar"), 32);
    let bar_constraints = HostPlacementConstraints::from(&bar_spec);
    assert_eq!(bar_constraints.widget_size, (0, 32));
    assert_eq!(bar_constraints.min_visible, 32);
    assert!(bar_constraints.work_area.is_none());

    // 2. floating HostSpec uses ContentPreferred (default min size) without manufacturing a fake static size
    let float_spec = HostSpec::floating(HostId::new("calendar_host"));
    let float_constraints = HostPlacementConstraints::from(&float_spec);
    assert_eq!(float_constraints.widget_size, (32, 32));
    assert_eq!(float_constraints.min_visible, 32);
    assert!(float_constraints.work_area.is_none());

    // 3. Exact size HostSpec preserves explicit width and height
    let mut exact_spec = HostSpec::floating(HostId::new("exact_host"));
    exact_spec.size = HostSize::Exact {
        width: 320,
        height: 240,
    };
    let exact_constraints = HostPlacementConstraints::from(&exact_spec);
    assert_eq!(exact_constraints.widget_size, (320, 240));

    // 4. FillOutput HostSpec preserves (0, 0)
    let mut fill_spec = HostSpec::floating(HostId::new("fill_host"));
    fill_spec.size = HostSize::FillOutput;
    let fill_constraints = HostPlacementConstraints::from(&fill_spec);
    assert_eq!(fill_constraints.widget_size, (0, 0));
}

#[test]
fn host_spec_output_height_to_placement_constraints_conversion() {
    use owlenshell::domain::host::HostSize;
    use owlenshell::placement::HostPlacementConstraints;

    let mut side_bar_spec = HostSpec::floating(HostId::new("side_bar"));
    side_bar_spec.size = HostSize::OutputHeight { width: 48 };
    let constraints = HostPlacementConstraints::from(&side_bar_spec);
    assert_eq!(constraints.widget_size, (48, 0));
    assert_eq!(constraints.min_visible, 32);
    assert!(constraints.work_area.is_none());
}

#[test]
fn clock_widget_controller_instantiation_and_action_dispatch() {
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::domain::widget::WidgetAction;
    use owlenshell::runtime::ClockWidgetController;

    let cal_host_id = HostId::new("calendar_popup");
    let controller =
        ClockWidgetController::new(WidgetId::new("clock_main"), Some(cal_host_id.clone()));

    assert_eq!(controller.id().as_str(), "clock_main");
    assert_eq!(controller.spec().title, "Clock");
    assert_eq!(controller.spec().kind, WidgetKind::Clock);
    assert_eq!(
        controller.update_interval(),
        std::time::Duration::from_secs(1)
    );

    // current_text returns a valid time string
    let text = controller.current_text();
    assert!(!text.is_empty());

    // handle_click emits generic WidgetAction::Command(HostCommand::Toggle)
    let action = controller.handle_click().expect("action must be emitted");
    assert_eq!(
        action,
        WidgetAction::Command(owlenshell::domain::widget::HostCommand::Toggle(cal_host_id))
    );
}

#[test]
fn clock_widget_controller_without_target_emits_no_action() {
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::runtime::ClockWidgetController;

    let controller = ClockWidgetController::new(WidgetId::new("standalone_clock"), None);
    assert_eq!(controller.handle_click(), None);
}

#[test]
fn calendar_widget_controller_state_and_content_generation() {
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::domain::widget::{HostCommand, WidgetAction};
    use owlenshell::runtime::CalendarWidgetController;

    let host_id = HostId::new("calendar_host");
    let controller = CalendarWidgetController::with_date(
        WidgetId::new("cal_widget"),
        host_id.clone(),
        2026,
        9, // September 2026 (Starts on Tuesday, 30 days)
    );

    assert_eq!(controller.id().as_str(), "cal_widget");
    assert_eq!(controller.spec().title, "Calendar");
    assert_eq!(controller.spec().kind, WidgetKind::Calendar);
    assert_eq!(controller.selected_year(), 2026);
    assert_eq!(controller.selected_month(), 9);
    assert_eq!(controller.month_title(), "September 2026");

    let rows = controller.week_rows();
    assert_eq!(rows.len(), 6);
    // Row 0: Monday=0, Tuesday=1, Wednesday=2, Thursday=3, Friday=4, Saturday=5, Sunday=6
    assert_eq!(rows[0].d0, 0);
    assert_eq!(rows[0].d1, 1);
    assert_eq!(rows[0].d2, 2);
    assert_eq!(rows[0].d6, 6);

    // Row 4: 28, 29, 30, 0, 0, 0, 0
    assert_eq!(rows[4].d0, 28);
    assert_eq!(rows[4].d1, 29);
    assert_eq!(rows[4].d2, 30);
    assert_eq!(rows[4].d3, 0);

    // Dismiss action emits generic HostCommand::Hide
    let action = controller.handle_dismiss();
    assert_eq!(action, WidgetAction::Command(HostCommand::Hide(host_id)));
}

#[test]
fn calendar_widget_controller_state_mutation_survives_independently() {
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::runtime::CalendarWidgetController;

    let mut controller = CalendarWidgetController::with_date(
        WidgetId::new("cal_widget"),
        HostId::new("calendar_host"),
        2026,
        8,
    );
    assert_eq!(controller.month_title(), "August 2026");

    controller.set_date(2026, 12);
    assert_eq!(controller.selected_year(), 2026);
    assert_eq!(controller.selected_month(), 12);
    assert_eq!(controller.month_title(), "December 2026");
}

#[test]
fn default_shell_spec_composition_invariants() {
    use owlenshell::domain::default_shell::{
        ShellSpec, DEFAULT_BAR_HOST_ID, DEFAULT_CALENDAR_HOST_ID, DEFAULT_CALENDAR_WIDGET_ID,
        DEFAULT_CLOCK_WIDGET_ID,
    };
    use owlenshell::domain::host::{HostLayer, HostMode, HostMovementPolicy, HostSize};
    use owlenshell::domain::ids::{HostId, WidgetId};
    use owlenshell::domain::placement::Anchor;
    use owlenshell::domain::widget::WidgetKind;

    let shell = ShellSpec::default_shell();

    // 1. Expected host declarations
    assert_eq!(shell.hosts.len(), 2);
    let bar_id = HostId::new(DEFAULT_BAR_HOST_ID);
    let cal_id = HostId::new(DEFAULT_CALENDAR_HOST_ID);

    let bar_host = shell.get_host(&bar_id).expect("bar host must exist");
    assert_eq!(bar_host.namespace, "owlenshell-bar");
    assert_eq!(bar_host.mode, HostMode::Cover);
    assert_eq!(bar_host.layer, HostLayer::Top);
    assert_eq!(bar_host.size, HostSize::OutputWidth { height: 32 });
    assert_eq!(bar_host.movement_policy, HostMovementPolicy::Fixed);
    assert!(bar_host.primary_milestone);
    assert!(bar_host.exit_on_close);

    let cal_host = shell.get_host(&cal_id).expect("calendar host must exist");
    assert_eq!(cal_host.namespace, "owlenshell-calendar");
    assert_eq!(cal_host.mode, HostMode::Float);
    assert_eq!(cal_host.layer, HostLayer::Top);
    assert_eq!(cal_host.size, HostSize::ContentPreferred);
    assert_eq!(cal_host.default_anchor, Anchor::TopRight);
    assert_eq!(cal_host.default_offset, (24, 40));
    assert_eq!(cal_host.movement_policy, HostMovementPolicy::Movable);
    assert!(!cal_host.primary_milestone);
    assert!(!cal_host.exit_on_close);

    // 2. Expected widget declarations
    assert_eq!(shell.widgets.len(), 2);
    let clock_id = WidgetId::new(DEFAULT_CLOCK_WIDGET_ID);
    let cal_widget_id = WidgetId::new(DEFAULT_CALENDAR_WIDGET_ID);

    let clock_widget = shell
        .get_widget(&clock_id)
        .expect("clock widget must exist");
    assert_eq!(clock_widget.kind, WidgetKind::Clock);
    assert_eq!(clock_widget.title, "Clock");

    let cal_widget = shell
        .get_widget(&cal_widget_id)
        .expect("calendar widget must exist");
    assert_eq!(cal_widget.kind, WidgetKind::Calendar);
    assert_eq!(cal_widget.title, "Calendar");

    // 3. Attachments
    assert_eq!(shell.attachments.len(), 2);
    assert!(shell
        .attachments
        .iter()
        .any(|a| a.host_id == bar_id && a.widget_id == clock_id));
    assert!(shell
        .attachments
        .iter()
        .any(|a| a.host_id == cal_id && a.widget_id == cal_widget_id));

    // 4. Initial visibility
    assert!(shell.is_initially_visible(&bar_id));
    assert!(!shell.is_initially_visible(&cal_id));
}
