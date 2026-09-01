use owlenshell::domain::ids::{HostId, OutputId};
use owlenshell::domain::output::OutputPreference;
use owlenshell::domain::placement::{Anchor, ResolveReason, SavedPlacement};
use owlenshell::model::SurfaceKind;
use owlenshell::persistence::Placements;
use owlenshell::platform::compositor::GenericAdapter;
use owlenshell::platform::output_manager::ProtocolOutput;
use owlenshell::platform::{
    OutputManager, OwlenPlatform, RenderBridge, SurfaceManager, TopologyGeneration,
};
use owlenshell::runtime::{Coordinator, DragState};
use owlenshell::ui::Calendar;
use std::cell::RefCell;
use std::rc::Rc;

fn make_proto_output(
    native_id: u32,
    name: &str,
    pos: (i32, i32),
    size: Option<(i32, i32)>,
) -> ProtocolOutput {
    ProtocolOutput {
        native_id,
        name: Some(name.into()),
        make: "TestVendor".into(),
        model: "TestModel".into(),
        physical_size: (500, 300),
        logical_position: Some(pos),
        logical_size: size,
        scale_factor: 1,
    }
}

#[test]
fn no_saved_intent_vs_saved_fallback_reason_separation() {
    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    // 1 active output: DP-1
    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let host_no_saved = HostId::new("no_saved");
    let host_with_missing_saved = HostId::new("with_saved");

    // Attach host without saved placement
    coordinator.attach_host(host_no_saved.clone(), SurfaceKind::Bar, None);

    // Attach host with saved placement targeting non-existent output "HDMI-A-1"
    let saved_missing = SavedPlacement {
        output: OutputPreference::new().with_name("HDMI-A-1"),
        anchor: Anchor::TopLeft,
        offset: (0, 0),
    };
    coordinator.attach_host(
        host_with_missing_saved.clone(),
        SurfaceKind::Calendar,
        Some(saved_missing),
    );

    coordinator.reconcile();

    let state_no_saved = coordinator.get_host(&host_no_saved).unwrap();
    let dec_no_saved = state_no_saved.current_decision.unwrap();
    assert_eq!(dec_no_saved.reason(), ResolveReason::Any);
    assert!(
        !dec_no_saved.is_fallback(),
        "No saved intent must not be marked as fallback"
    );

    let state_with_saved = coordinator.get_host(&host_with_missing_saved).unwrap();
    let dec_with_saved = state_with_saved.current_decision.unwrap();
    assert_eq!(dec_with_saved.reason(), ResolveReason::Any);
    assert!(
        dec_with_saved.is_fallback(),
        "Missing saved intent target must be marked as fallback"
    );
}

#[test]
fn stale_topology_decision_re_resolves() {
    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    // Gen 1: Only DP-1 exists
    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let host_id = HostId::new("calendar");
    let saved_hdmi = SavedPlacement {
        output: OutputPreference::new().with_name("HDMI-A-1"),
        anchor: Anchor::TopRight,
        offset: (20, 20),
    };
    coordinator.attach_host(host_id.clone(), SurfaceKind::Calendar, Some(saved_hdmi));

    coordinator.reconcile();

    let state1 = coordinator.get_host(&host_id).unwrap();
    assert_eq!(state1.decision_generation, Some(TopologyGeneration::new(1)));
    let dec1 = state1.current_decision.unwrap();
    assert_eq!(dec1.target_output_id(), Some(OutputId::new(1)));
    assert!(dec1.is_fallback());

    // Gen 2: Preferred HDMI-A-1 appears!
    output_manager.borrow_mut().sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080))),
        make_proto_output(2, "HDMI-A-1", (1920, 0), Some((1920, 1080))),
    ]);

    // Reconcile must recognize decision_generation (1) != current_generation (2) and re-resolve to home output
    coordinator.reconcile();

    let state2 = coordinator.get_host(&host_id).unwrap();
    assert_eq!(state2.decision_generation, Some(TopologyGeneration::new(2)));
    let dec2 = state2.current_decision.unwrap();
    assert_eq!(dec2.target_output_id(), Some(OutputId::new(2)));
    assert_eq!(dec2.reason(), ResolveReason::ExactName);
    assert!(
        !dec2.is_fallback(),
        "Home output resolution is not fallback"
    );
}

#[test]
fn fallback_does_not_persist_only_explicit_drop_persists() {
    let bridge = RenderBridge::new();
    let platform = OwlenPlatform::new(bridge.clone());
    let _ = slint::platform::set_platform(Box::new(platform));

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));

    // Gen 1: Only DP-1 exists
    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let mut initial_store = Placements::default();
    let host_id = HostId::new("calendar");
    let home_saved = SavedPlacement {
        output: OutputPreference::new().with_name("HDMI-A-1"),
        anchor: Anchor::TopRight,
        offset: (20, 20),
    };
    initial_store.set(host_id.clone(), home_saved.clone());

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        bridge.clone(),
        initial_store,
    );

    // Create Slint component through RenderBridge to register the adapter
    let _cal = bridge
        .with_host_creation(host_id.clone(), Calendar::new)
        .expect("calendar creation");

    coordinator.attach_host(
        host_id.clone(),
        SurfaceKind::Calendar,
        Some(home_saved.clone()),
    );

    // Reconcile falls back to DP-1
    coordinator.reconcile();

    // Invariant: Fallback resolution must NEVER overwrite saved home intent in store
    let store_after_reconcile = coordinator.placements_store();
    assert_eq!(
        store_after_reconcile.get(&host_id),
        Some(&home_saved),
        "Fallback must not overwrite stored placement"
    );

    // Execute explicit drop on DP-1
    let drag = DragState {
        host_id: host_id.clone(),
        grab: (10.0, 10.0),
        start_margins: (0, 20, 20, 0),
        anchor: Anchor::TopRight,
        output_id: OutputId::new(1),
    };

    coordinator.finish_drag(&drag, (50.0, 60.0));

    // Invariant: Explicit completed drop updates persistence
    let store_after_drop = coordinator.placements_store();
    let updated_saved = store_after_drop.get(&host_id).unwrap();
    assert_eq!(
        updated_saved.output.name.as_deref(),
        Some("DP-1"),
        "Explicit drop must persist the new target output"
    );
}

#[test]
fn hidden_logical_state_survives_native_destruction() {
    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let host_id = HostId::new("calendar");
    let saved = SavedPlacement {
        output: OutputPreference::new().with_name("DP-1"),
        anchor: Anchor::TopRight,
        offset: (40, 50),
    };

    coordinator.attach_host(host_id.clone(), SurfaceKind::Calendar, Some(saved.clone()));

    // Create fake native instance token
    let _token = surface_manager
        .borrow_mut()
        .generation_registry_mut_for_test()
        .allocate(host_id.clone());

    // Hide host
    coordinator.hide_host(&host_id);

    // Native token is retired in SurfaceManager
    assert_eq!(surface_manager.borrow().current_token(&host_id), None);

    // Logical state in coordinator survives with visibility false and saved placement intact
    let host_state = coordinator.get_host(&host_id).unwrap();
    assert!(!host_state.visible);
    assert_eq!(host_state.saved_placement, Some(saved));
}

#[test]
fn protocol_isolation_invariant() {
    let source_path = "src/runtime/coordinator.rs";
    let content = std::fs::read_to_string(source_path).expect("read coordinator.rs");
    let forbidden = [
        "wayland_client",
        "smithay_client_toolkit",
        "calloop_wayland_source",
        "WlSurface",
        "WlOutput",
        "LayerSurface",
    ];
    for term in forbidden {
        assert!(
            !content.contains(term),
            "Coordinator must not contain raw Wayland/SCTK protocol type '{term}'"
        );
    }
}

#[test]
fn calendar_attach_resolves_without_topology_dirty() {
    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let host_id = HostId::new("calendar");
    // Attach host: sets current_decision to None
    coordinator.attach_host(host_id.clone(), SurfaceKind::Calendar, None);

    let state = coordinator.get_host(&host_id).unwrap();
    assert!(state.visible);
    assert!(state.current_decision.is_none());

    // Without any topology dirty event or manager mutation, Coordinator reconciles pending resolution
    // Simulating the pump condition: is_topology_dirty = false, pending_resolution = true
    let pending_resolution = {
        let h = coordinator.get_host(&host_id).unwrap();
        h.visible && h.current_decision.is_none()
    };
    assert!(
        pending_resolution,
        "Newly attached host must have pending resolution"
    );

    // Reconcile triggered by pending_resolution
    coordinator.reconcile();

    let resolved_state = coordinator.get_host(&host_id).unwrap();
    assert!(
        resolved_state.current_decision.is_some(),
        "Host must be resolved and ready to attach without topology change"
    );
    assert_eq!(
        resolved_state.current_decision.unwrap().target_output_id(),
        Some(OutputId::new(1))
    );
}

#[test]
fn cross_output_drop_re_resolves_without_topology_dirty() {
    let bridge = RenderBridge::new();
    let platform = OwlenPlatform::new(bridge.clone());
    let _ = slint::platform::set_platform(Box::new(platform));

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));

    // 2 outputs: DP-1 and HDMI-A-1
    output_manager.borrow_mut().sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080))),
        make_proto_output(2, "HDMI-A-1", (1920, 0), Some((1920, 1080))),
    ]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        bridge.clone(),
        Placements::default(),
    );

    let host_id = HostId::new("calendar");
    let _cal = bridge
        .with_host_creation(host_id.clone(), Calendar::new)
        .expect("calendar creation");

    let initial_saved = SavedPlacement {
        output: OutputPreference::new().with_name("DP-1"),
        anchor: Anchor::TopLeft,
        offset: (10, 10),
    };
    coordinator.attach_host(host_id.clone(), SurfaceKind::Calendar, Some(initial_saved));
    coordinator.reconcile();

    let state1 = coordinator.get_host(&host_id).unwrap();
    assert_eq!(
        state1.current_decision.as_ref().unwrap().target_output_id(),
        Some(OutputId::new(1))
    );

    // Active drag starting on DP-1 (output 1)
    let drag = DragState {
        host_id: host_id.clone(),
        grab: (10.0, 10.0),
        start_margins: (10, 0, 0, 10),
        anchor: Anchor::TopLeft,
        output_id: OutputId::new(1),
    };

    // Drop across into HDMI-A-1 (global X > 1920)
    // pointer = (1950.0, 50.0)
    coordinator.finish_drag(&drag, (1950.0, 50.0));

    // After finish_drag:
    // 1. Saved placement updated to HDMI-A-1
    let store = coordinator.placements_store();
    let saved_after_drag = store.get(&host_id).unwrap();
    assert_eq!(saved_after_drag.output.name.as_deref(), Some("HDMI-A-1"));

    // 2. Decision is invalidated (current_decision = None)
    let state_after_drag = coordinator.get_host(&host_id).unwrap();
    assert!(
        state_after_drag.current_decision.is_none(),
        "Decision must be invalidated on cross-output move"
    );

    // 3. Topology is not dirty, but pending_resolution is true
    let pending_resolution =
        state_after_drag.visible && state_after_drag.current_decision.is_none();
    assert!(pending_resolution);

    // 4. Reconcile re-resolves and targets HDMI-A-1 (output 2)
    coordinator.reconcile();

    let state_final = coordinator.get_host(&host_id).unwrap();
    let dec_final = state_final.current_decision.unwrap();
    assert_eq!(
        dec_final.target_output_id(),
        Some(OutputId::new(2)),
        "Host must target new output after cross-output drop"
    );
    assert_eq!(dec_final.reason(), ResolveReason::ExactName);
    assert!(!dec_final.is_fallback());
}

#[test]
fn handle_height_boundary_is_28_pixels() {
    assert_eq!(owlenshell::model::HANDLE_HEIGHT, 28.0);
    assert_eq!(owlenshell::runtime::HANDLE_HEIGHT, 28.0);

    let inside_pointer_y = 27.9;
    let outside_pointer_y = 28.0;
    assert!(inside_pointer_y < owlenshell::runtime::HANDLE_HEIGHT);
    assert!(outside_pointer_y >= owlenshell::runtime::HANDLE_HEIGHT);
}

#[test]
fn coordinator_generic_host_and_widget_attachment_lifecycle() {
    use owlenshell::domain::host::{HostMovementPolicy, HostSize, HostSpec};
    use owlenshell::domain::ids::WidgetId;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let bar_spec = HostSpec::top_bar(HostId::new("primary_bar"), 32);
    coordinator.attach_host_spec(bar_spec.clone(), None);

    // 1. Host carries HostSpec in HostState
    let host_state = coordinator.get_host(&HostId::new("primary_bar")).unwrap();
    assert_eq!(host_state.spec.id.as_str(), "primary_bar");
    assert_eq!(host_state.spec.size, HostSize::OutputWidth { height: 32 });
    assert!(host_state.spec.primary_milestone);
    assert_eq!(host_state.spec.movement_policy, HostMovementPolicy::Fixed);

    // 2. Generic widget attachment
    let clock_id = WidgetId::new("clock_widget");
    let battery_id = WidgetId::new("battery_widget");
    assert!(coordinator.attach_widget(&HostId::new("primary_bar"), clock_id.clone()));
    assert!(coordinator.attach_widget(&HostId::new("primary_bar"), battery_id.clone()));

    // 3. attached_widgets is the canonical containment relation
    let attached = coordinator
        .attached_widgets(&HostId::new("primary_bar"))
        .unwrap();
    assert_eq!(attached, vec![clock_id.clone(), battery_id.clone()]);

    // 4. Detachment
    assert!(coordinator.detach_widget(&HostId::new("primary_bar"), &clock_id));
    let attached_after = coordinator
        .attached_widgets(&HostId::new("primary_bar"))
        .unwrap();
    assert_eq!(attached_after, vec![battery_id]);
}

#[test]
fn coordinator_reconcile_derives_placement_constraints_from_host_spec() {
    use owlenshell::domain::host::HostSpec;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    output_manager
        .borrow_mut()
        .sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let bar_spec = HostSpec::top_bar(HostId::new("bar"), 32);
    let float_spec = HostSpec::floating(HostId::new("float_popup"));

    coordinator.attach_host_spec(bar_spec, None);
    coordinator.attach_host_spec(float_spec, None);

    coordinator.reconcile();

    let bar_state = coordinator.get_host(&HostId::new("bar")).unwrap();
    let float_state = coordinator.get_host(&HostId::new("float_popup")).unwrap();

    assert!(bar_state.current_decision.is_some());
    assert!(float_state.current_decision.is_some());
}

#[test]
fn coordinator_lifecycle_exit_on_close_and_primary_milestone_are_independent() {
    use owlenshell::domain::host::HostSpec;
    use owlenshell::platform::{PlatformLifecycleEvent, SurfaceGeneration, SurfaceToken};

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    // Host 1: primary_milestone = true, exit_on_close = false
    let mut host1_spec = HostSpec::top_bar(HostId::new("milestone_only"), 32);
    host1_spec.exit_on_close = false;
    coordinator.attach_host_spec(host1_spec, None);

    // Host 2: primary_milestone = false, exit_on_close = true
    let mut host2_spec = HostSpec::floating(HostId::new("exit_only"));
    host2_spec.exit_on_close = true;
    coordinator.attach_host_spec(host2_spec, None);

    // 1. FrameRendered on host1 (primary_milestone = true) fires first-frame / shell-ready
    assert!(!coordinator.first_frame_fired());
    coordinator.process_lifecycle_event(PlatformLifecycleEvent::FrameRendered {
        host_id: HostId::new("milestone_only"),
        token: SurfaceToken::new(HostId::new("milestone_only"), SurfaceGeneration::INITIAL),
    });
    assert!(coordinator.first_frame_fired());

    // FrameRendered on host2 (primary_milestone = false) does not affect state
    assert!(!coordinator.should_exit());

    // 2. Closed on host1 (exit_on_close = false) does NOT request exit
    coordinator.process_lifecycle_event(PlatformLifecycleEvent::Closed {
        host_id: HostId::new("milestone_only"),
        token: SurfaceToken::new(HostId::new("milestone_only"), SurfaceGeneration::INITIAL),
    });
    assert!(
        !coordinator.should_exit(),
        "Host with exit_on_close=false must not exit"
    );

    // 3. Closed on host2 (exit_on_close = true) requests exit
    coordinator.process_lifecycle_event(PlatformLifecycleEvent::Closed {
        host_id: HostId::new("exit_only"),
        token: SurfaceToken::new(HostId::new("exit_only"), SurfaceGeneration::INITIAL),
    });
    assert!(
        coordinator.should_exit(),
        "Host with exit_on_close=true must exit"
    );
}

#[test]
fn coordinator_dispatch_action_toggles_target_host_visibility() {
    use owlenshell::domain::host::HostSpec;
    use owlenshell::domain::widget::WidgetAction;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let popup_id = HostId::new("test_popup");
    let popup_spec = HostSpec::floating(popup_id.clone());
    coordinator.attach_host_spec(popup_spec, None);

    // Initial state: visible = true
    assert!(coordinator.get_host(&popup_id).unwrap().visible);

    // 1. Dispatch ToggleHost -> host becomes hidden
    coordinator.dispatch_action(WidgetAction::ToggleHost(popup_id.clone()));
    assert!(!coordinator.get_host(&popup_id).unwrap().visible);

    // 2. Dispatch ToggleHost again -> host becomes visible
    coordinator.dispatch_action(WidgetAction::ToggleHost(popup_id.clone()));
    assert!(coordinator.get_host(&popup_id).unwrap().visible);
}

#[test]
fn coordinator_first_attachment_vs_subsequent_toggle_semantics() {
    use owlenshell::domain::widget::WidgetAction;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let cal_id = HostId::new("calendar");

    // 1. Initial state: host absent
    assert!(coordinator.get_host(&cal_id).is_none());

    // 2. First click on absent host: attachment establishes visible = true
    coordinator.attach_host(cal_id.clone(), SurfaceKind::Calendar, None);
    let state_after_attach = coordinator.get_host(&cal_id).expect("host must exist");
    assert!(
        state_after_attach.visible,
        "Initial attach must set visible = true"
    );

    // 3. Second click: dispatch_action(ToggleHost) inverts visible from true -> false
    coordinator.dispatch_action(WidgetAction::ToggleHost(cal_id.clone()));
    let state_after_first_toggle = coordinator.get_host(&cal_id).expect("host must exist");
    assert!(
        !state_after_first_toggle.visible,
        "First toggle must set visible = false"
    );

    // 4. Third click: dispatch_action(ToggleHost) inverts visible from false -> true
    coordinator.dispatch_action(WidgetAction::ToggleHost(cal_id.clone()));
    let state_after_second_toggle = coordinator.get_host(&cal_id).expect("host must exist");
    assert!(
        state_after_second_toggle.visible,
        "Second toggle must set visible = true"
    );
}

#[test]
fn calendar_widget_state_survives_host_hide_show_and_native_retirement() {
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::domain::widget::WidgetAction;
    use owlenshell::runtime::CalendarWidgetController;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let cal_host_id = HostId::new("calendar_host");
    let mut calendar_controller = CalendarWidgetController::with_date(
        WidgetId::new("calendar_main"),
        cal_host_id.clone(),
        2026,
        11, // November 2026
    );

    // 1. Initial attachment
    coordinator.attach_host(cal_host_id.clone(), SurfaceKind::Calendar, None);
    coordinator.attach_widget(&cal_host_id, calendar_controller.id().clone());

    let attached = coordinator.attached_widgets(&cal_host_id).unwrap();
    assert_eq!(attached, vec![WidgetId::new("calendar_main")]);
    assert_eq!(calendar_controller.month_title(), "November 2026");

    // 2. Hide host via generic HideHost action
    coordinator.dispatch_action(calendar_controller.handle_dismiss());
    assert!(!coordinator.get_host(&cal_host_id).unwrap().visible);

    // 3. User navigates / mutates controller state while hidden
    calendar_controller.set_date(2026, 12);

    // 4. Re-show host via generic ToggleHost action
    coordinator.dispatch_action(WidgetAction::ToggleHost(cal_host_id.clone()));
    assert!(coordinator.get_host(&cal_host_id).unwrap().visible);

    // 5. Invariants: attached WidgetId survives, controller content state survives
    let attached_after = coordinator.attached_widgets(&cal_host_id).unwrap();
    assert_eq!(attached_after, vec![WidgetId::new("calendar_main")]);
    assert_eq!(calendar_controller.month_title(), "December 2026");
}

#[test]
fn coordinator_apply_shell_spec_and_generic_command_execution() {
    use owlenshell::domain::default_shell::{
        ShellSpec, DEFAULT_BAR_HOST_ID, DEFAULT_CALENDAR_HOST_ID, DEFAULT_CALENDAR_WIDGET_ID,
        DEFAULT_CLOCK_WIDGET_ID,
    };
    use owlenshell::domain::ids::WidgetId;
    use owlenshell::domain::widget::HostCommand;

    let output_manager = Rc::new(RefCell::new(OutputManager::new(Box::new(GenericAdapter))));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let coordinator = Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::default(),
    );

    let shell_spec = ShellSpec::default_shell();
    coordinator.apply_shell_spec(&shell_spec);

    let bar_id = HostId::new(DEFAULT_BAR_HOST_ID);
    let cal_id = HostId::new(DEFAULT_CALENDAR_HOST_ID);
    let clock_id = WidgetId::new(DEFAULT_CLOCK_WIDGET_ID);
    let cal_widget_id = WidgetId::new(DEFAULT_CALENDAR_WIDGET_ID);

    // 1. Invariants after apply_shell_spec:
    // Bar host is visible with clock widget attached
    let bar_state = coordinator.get_host(&bar_id).expect("bar host must exist");
    assert!(bar_state.visible);
    assert_eq!(bar_state.attached_widgets, vec![clock_id]);

    // Calendar host is logically present but hidden, with calendar widget attached
    let cal_state = coordinator.get_host(&cal_id).expect("cal host must exist");
    assert!(!cal_state.visible, "Calendar must be initially hidden");
    assert_eq!(cal_state.attached_widgets, vec![cal_widget_id]);

    // 2. Generic HostCommand::Toggle execution (e.g. clock click)
    coordinator.execute_command(HostCommand::Toggle(cal_id.clone()));
    assert!(
        coordinator.get_host(&cal_id).unwrap().visible,
        "Toggle must show hidden calendar"
    );

    // 3. Generic HostCommand::Toggle again
    coordinator.execute_command(HostCommand::Toggle(cal_id.clone()));
    assert!(
        !coordinator.get_host(&cal_id).unwrap().visible,
        "Second toggle must hide calendar"
    );

    // 4. Generic HostCommand::Show
    coordinator.execute_command(HostCommand::Show(cal_id.clone()));
    assert!(coordinator.get_host(&cal_id).unwrap().visible);

    // 5. Generic HostCommand::Hide (e.g. calendar dismiss)
    coordinator.execute_command(HostCommand::Hide(cal_id.clone()));
    assert!(!coordinator.get_host(&cal_id).unwrap().visible);
}
