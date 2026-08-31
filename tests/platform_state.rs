use owlenshell::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use owlenshell::domain::ids::{HostId, OutputId};
use owlenshell::domain::output::OutputIdentity;
use owlenshell::domain::placement::{Anchor, PlacedSurface, ResolveReason};
use owlenshell::platform::output_manager::{
    NewOutput, OutputRegistry, TopologyEvent, TopologyGeneration,
};
use owlenshell::platform::surface_manager::{
    SurfaceCommand, SurfaceGeneration, SurfaceGenerationRegistry, SurfaceLifecycleEvent,
    SurfaceSpec, SurfaceToken,
};
use owlenshell::platform::{PlatformError, PlatformEvent};

fn make_new_output(name: &str, logical_size: Option<(i32, i32)>) -> NewOutput {
    NewOutput {
        name: Some(name.into()),
        identity: OutputIdentity {
            make: Some("TestVendor".into()),
            model: Some("TestModel".into()),
            physical_size_mm: Some((600, 340)),
            serial: Some(format!("serial-{name}")),
        },
        logical_position: Some((0, 0)),
        logical_size,
        logical_role: None,
        scale: 1,
        preferred: false,
    }
}

#[test]
fn output_id_allocation_exclusive_and_monotonic() {
    let mut registry = OutputRegistry::new();
    let id1 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    let id2 = registry.add(make_new_output("DP-2", Some((2560, 1440))));
    let id3 = registry.add(make_new_output("HDMI-1", Some((1920, 1080))));

    assert_eq!(id1, OutputId::new(1));
    assert_eq!(id2, OutputId::new(2));
    assert_eq!(id3, OutputId::new(3));
    assert!(id2 > id1);
    assert!(id3 > id2);
}

#[test]
fn output_removal_and_readd_allocates_fresh_id_and_never_reuses_retired_id() {
    let mut registry = OutputRegistry::new();
    let id1 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    let id2 = registry.add(make_new_output("DP-2", Some((2560, 1440))));

    // Disconnect/remove id1
    let removed = registry.remove(id1);
    assert_eq!(removed.map(|o| o.id), Some(id1));
    assert!(registry.is_retired(id1));
    assert!(!registry.is_active(id1));

    // Re-add the same physical monitor (DP-1): must allocate a brand new OutputId
    let id3 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    assert_ne!(id3, id1, "disconnected OutputId must never be reused");
    assert_eq!(id3, OutputId::new(3));
    assert!(registry.is_retired(id1));
    assert!(registry.is_active(id3));
    assert!(registry.is_active(id2));

    // Retired ID cannot be updated through update API
    let mut fake_snapshot = make_new_output("DP-1", Some((1920, 1080))).into_snapshot(id1);
    fake_snapshot.scale = 2;
    let update_res = registry.update(fake_snapshot);
    assert!(!update_res, "retired ID cannot be updated or revived");
}

#[test]
fn output_update_preserves_id_and_advances_generation_only_on_change() {
    let mut registry = OutputRegistry::new();
    let id1 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    assert_eq!(registry.generation(), TopologyGeneration::new(1));

    let mut snapshot = registry.get(id1).cloned().unwrap();
    assert_eq!(snapshot.id, id1);

    // Identical update: generation does NOT spuriously advance
    let changed_same = registry.update(snapshot.clone());
    assert!(!changed_same);
    assert_eq!(registry.generation(), TopologyGeneration::new(1));

    // Modified update: generation advances, ID is preserved
    snapshot.scale = 2;
    let changed_mod = registry.update(snapshot);
    assert!(changed_mod);
    assert_eq!(registry.generation(), TopologyGeneration::new(2));
    assert_eq!(registry.get(id1).unwrap().scale, 2);
    assert_eq!(registry.get(id1).unwrap().id, id1);
}

#[test]
fn output_removal_reflected_in_canonical_snapshot_before_consumption() {
    let mut registry = OutputRegistry::new();
    let id1 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    let id2 = registry.add(make_new_output("HDMI-1", Some((1920, 1080))));

    let snapshot_before = registry.snapshot();
    assert_eq!(snapshot_before.len(), 2);
    assert!(snapshot_before.get(id1).is_some());

    // Synchronously remove id1
    let removed = registry.remove(id1);
    assert!(removed.is_some());

    // Canonical snapshot immediately reflects removal
    let snapshot_after = registry.snapshot();
    assert_eq!(snapshot_after.len(), 1);
    assert!(
        snapshot_after.get(id1).is_none(),
        "removed output must be completely absent from canonical snapshot"
    );
    assert!(snapshot_after.get(id2).is_some());
}

#[test]
fn placement_capable_outputs_filters_incomplete_geometry() {
    let mut registry = OutputRegistry::new();
    let id1 = registry.add(make_new_output("DP-1", Some((1920, 1080))));
    let _id2 = registry.add(make_new_output("DP-2", None));

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.len(), 2);

    let placement_capable: Vec<_> = snapshot.placement_capable_outputs().collect();
    assert_eq!(placement_capable.len(), 1);
    assert_eq!(placement_capable[0].id, id1);
}

#[test]
fn surface_token_validation_rejects_wrong_host_id() {
    let mut registry = SurfaceGenerationRegistry::new();
    let host_a = HostId::new("host-a");
    let host_b = HostId::new("host-b");

    // Both hosts active with generation 1
    let token_a = registry.allocate(host_a.clone());
    let token_b = registry.allocate(host_b.clone());

    assert_eq!(token_a.generation, SurfaceGeneration::new(1));
    assert_eq!(token_b.generation, SurfaceGeneration::new(1));

    // Token for host-a must validate for host-a, and fail for host-b
    assert!(registry.validate(&host_a, &token_a));
    assert!(
        !registry.validate(&host_b, &token_a),
        "host-a token must NOT validate when checked for host-b"
    );

    // Token for host-b must validate for host-b, and fail for host-a
    assert!(registry.validate(&host_b, &token_b));
    assert!(
        !registry.validate(&host_a, &token_b),
        "host-b token must NOT validate when checked for host-a"
    );

    // Stale token for host-a after recreation
    let token_a_gen2 = registry.recreate(&host_a);
    assert_eq!(token_a_gen2.generation, SurfaceGeneration::new(2));
    assert!(registry.validate(&host_a, &token_a_gen2));
    assert!(
        !registry.validate(&host_a, &token_a),
        "stale host-a generation 1 token must fail"
    );

    // Retired token for host-b after hide
    let retired_b = registry.hide(&host_b).unwrap();
    assert!(
        !registry.validate(&host_b, &retired_b),
        "retired token must not validate"
    );
    assert!(
        !registry.validate(&host_b, &token_b),
        "token_b must not validate after retirement"
    );
}

#[test]
fn repeated_surface_retirement_and_monotonic_progression() {
    let mut registry = SurfaceGenerationRegistry::new();
    let cal_id = HostId::new("calendar");

    // 1. Allocate initial generation 1
    let token1 = registry.allocate(cal_id.clone());
    assert_eq!(token1.generation, SurfaceGeneration::new(1));
    assert!(registry.validate(&cal_id, &token1));

    // 2. Retire generation 1
    let retired1 = registry.retire(&cal_id);
    assert_eq!(retired1, Some(token1.clone()));
    assert!(!registry.validate(&cal_id, &token1));
    assert_eq!(registry.current_token(&cal_id), None);
    assert_eq!(registry.current_generation(&cal_id), None);
    assert!(!registry.has_active_surface(&cal_id));

    // 3. Retire again when already retired -> returns None cleanly
    let retired_again = registry.retire(&cal_id);
    assert_eq!(retired_again, None);

    // 4. Allocate again -> monotonically advances to generation 2
    let token2 = registry.allocate(cal_id.clone());
    assert_eq!(token2.generation, SurfaceGeneration::new(2));
    assert!(registry.validate(&cal_id, &token2));
    assert!(!registry.validate(&cal_id, &token1));

    // 5. Recreate -> advances to generation 3
    let token3 = registry.recreate(&cal_id);
    assert_eq!(token3.generation, SurfaceGeneration::new(3));
    assert!(registry.validate(&cal_id, &token3));
    assert!(!registry.validate(&cal_id, &token2));
    assert!(!registry.validate(&cal_id, &token1));
}

#[test]
fn typed_surface_and_platform_commands_and_events() {
    let host_id = HostId::new("bar");
    let spec = SurfaceSpec {
        namespace: "owlenshell-bar".into(),
        layer: HostLayer::Top,
        mode: HostMode::Cover,
        size: HostSize::FillOutput,
        exclusive_zone: ExclusiveZonePolicy::Auto,
        keyboard_interactivity: KeyboardInteractivityPolicy::None,
    };
    let placement = PlacedSurface {
        target_output_id: OutputId::new(1),
        anchor: Anchor::TopLeft,
        offset: (0, 0),
        margins: (0, 0, 0, 0),
        reason: ResolveReason::ExactName,
        is_fallback: false,
        is_reachable: true,
    };

    let cmd = SurfaceCommand::EnsureVisible {
        host_id: host_id.clone(),
        spec,
        placement,
    };

    match cmd {
        SurfaceCommand::EnsureVisible { host_id: h, .. } => assert_eq!(h, host_id),
        _ => panic!("unexpected command variant"),
    }

    let token = SurfaceToken::new(host_id.clone(), SurfaceGeneration::new(1));
    let lifecycle_event = SurfaceLifecycleEvent::Configured {
        token: token.clone(),
        width: 1920,
        height: 32,
    };

    let platform_event = PlatformEvent::Surface(lifecycle_event);
    match platform_event {
        PlatformEvent::Surface(SurfaceLifecycleEvent::Configured {
            token: t,
            width,
            height,
        }) => {
            assert_eq!(t, token);
            assert_eq!(width, 1920);
            assert_eq!(height, 32);
        }
        _ => panic!("unexpected platform event"),
    }

    let err = PlatformError::StaleGeneration {
        token: token.clone(),
    };
    assert_eq!(
        format!("{err}"),
        format!("stale surface generation: {token}")
    );
}

#[test]
fn topology_events_and_generation_display() {
    let gen = TopologyGeneration::new(42);
    assert_eq!(format!("{gen}"), "topology_gen#42");
    assert_eq!(gen.raw(), 42);

    let out = make_new_output("DP-1", Some((1920, 1080))).into_snapshot(OutputId::new(1));
    let event = TopologyEvent::OutputAdded(out.clone());
    let platform_event = PlatformEvent::Topology(event);

    match platform_event {
        PlatformEvent::Topology(TopologyEvent::OutputAdded(s)) => assert_eq!(s, out),
        _ => panic!("unexpected event"),
    }
}
