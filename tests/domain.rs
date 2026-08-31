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
use owlenshell::domain::widget::{WidgetSpec, WidgetState};

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
    };

    assert_eq!(spec.mode, HostMode::Float);
    assert_eq!(spec.layer, HostLayer::Top);
    assert_eq!(spec.movement_policy, HostMovementPolicy::Movable);

    let mut state = HostState::new(spec.id.clone());
    assert!(!state.visible);
    state.attach_widget(WidgetId::new("generic-widget-1"));
    assert_eq!(state.attached_widgets().len(), 1);
    assert_eq!(state.attached_widgets()[0].as_str(), "generic-widget-1");

    assert!(state.detach_widget(&WidgetId::new("generic-widget-1")));
    assert!(state.attached_widgets().is_empty());
}

#[test]
fn widget_spec_and_state_are_decoupled_from_host_policy() {
    let widget_spec = WidgetSpec {
        id: WidgetId::new("weather-widget"),
        title: "Weather".into(),
        enabled_by_default: true,
    };
    assert_eq!(widget_spec.id.as_str(), "weather-widget");

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
