use owlenshell::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use owlenshell::domain::ids::{HostId, OutputId};
use owlenshell::domain::output::{OutputIdentity, OutputPreference, OutputSnapshot};
use owlenshell::domain::placement::{Anchor, PlacedSurface, ResolveReason};
use owlenshell::placement::{resolve, PlacementContext};
use owlenshell::platform::compositor::{CompositorAdapter, CompositorOutputFacts, GenericAdapter};
use owlenshell::platform::output_manager::{
    NewOutput, OutputManager, OutputRegistry, ProtocolOutput, TopologyEvent, TopologyGeneration,
};
use owlenshell::platform::surface_manager::{
    SurfaceCommand, SurfaceGeneration, SurfaceGenerationRegistry, SurfaceLifecycleEvent,
    SurfaceManager, SurfaceSpec, SurfaceToken,
};
use owlenshell::platform::{PlatformError, PlatformEvent};
use std::cell::RefCell;

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

fn make_proto_output(
    native_id: u32,
    name: &str,
    pos: (i32, i32),
    size: Option<(i32, i32)>,
) -> ProtocolOutput {
    ProtocolOutput {
        native_id,
        name: Some(name.into()),
        make: "TestMake".into(),
        model: "TestModel".into(),
        physical_size: (600, 340),
        logical_position: Some(pos),
        logical_size: size,
        scale_factor: 1,
    }
}

// ---------------------------------------------------------------------------
// 1. Geometry Readiness Tests
// ---------------------------------------------------------------------------

#[test]
fn geometry_readiness_adversarial_matrix() {
    let test_cases = vec![
        (None, false, "None logical size"),
        (Some((0, 0)), false, "(0, 0) zero dimensions"),
        (Some((0, 1080)), false, "(0, positive) zero width"),
        (Some((1920, 0)), false, "(positive, 0) zero height"),
        (Some((-1920, 1080)), false, "negative width"),
        (Some((1920, -1080)), false, "negative height"),
        (Some((-1920, -1080)), false, "negative width and height"),
        (Some((1920, 1080)), true, "positive width and height"),
    ];

    for (size, expected_capable, desc) in test_cases {
        let snap = OutputSnapshot {
            id: OutputId::new(1),
            name: Some("DP-1".into()),
            identity: OutputIdentity::default(),
            logical_position: Some((0, 0)),
            logical_size: size,
            logical_role: None,
            scale: 1,
            preferred: false,
        };
        assert_eq!(
            snap.is_placement_capable(),
            expected_capable,
            "failed case: {desc}"
        );
    }
}

#[test]
fn mixed_geometry_readiness_filters_only_valid_outputs() {
    let mut mgr = OutputManager::new(Box::new(GenericAdapter));
    let proto_valid = make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)));
    let proto_zero = make_proto_output(2, "DP-2", (1920, 0), Some((0, 0)));
    let proto_none = make_proto_output(3, "HDMI-1", (3840, 0), None);
    let proto_neg = make_proto_output(4, "HDMI-2", (5760, 0), Some((-1920, 1080)));

    mgr.sync_protocol_facts(&[proto_valid, proto_zero, proto_none, proto_neg]);

    let snapshot = mgr.snapshot();
    assert_eq!(
        snapshot.len(),
        4,
        "all outputs present in canonical inventory for diagnostics"
    );

    let capable: Vec<_> = snapshot.placement_capable_outputs().collect();
    assert_eq!(capable.len(), 1);
    assert_eq!(capable[0].name.as_deref(), Some("DP-1"));
    assert_eq!(capable[0].logical_size, Some((1920, 1080)));
}

#[test]
fn all_invalid_geometry_defers_placement_resolution() {
    let mut mgr = OutputManager::new(Box::new(GenericAdapter));
    let proto_zero = make_proto_output(1, "DP-1", (0, 0), Some((0, 0)));
    let proto_none = make_proto_output(2, "DP-2", (1920, 0), None);

    mgr.sync_protocol_facts(&[proto_zero, proto_none]);

    let snapshot = mgr.snapshot();
    let placement_capable: Vec<OutputSnapshot> =
        snapshot.placement_capable_outputs().cloned().collect();
    assert!(placement_capable.is_empty());

    let decision = resolve(
        &OutputPreference::new().with_name("DP-1"),
        &placement_capable,
        &PlacementContext::default(),
    );
    assert_eq!(decision.reason(), ResolveReason::Deferred);
    assert!(decision.is_deferred());
    assert_eq!(decision.target_output_id(), None);
}

// ---------------------------------------------------------------------------
// 2. Compositor Enrichment Freshness & Safety Tests
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct DynamicMockAdapter {
    facts: std::rc::Rc<RefCell<Vec<CompositorOutputFacts>>>,
    preferred: std::rc::Rc<RefCell<Option<String>>>,
    fail_query: std::rc::Rc<RefCell<bool>>,
}

impl DynamicMockAdapter {
    fn new(facts: Vec<CompositorOutputFacts>, preferred: Option<String>) -> Self {
        Self {
            facts: std::rc::Rc::new(RefCell::new(facts)),
            preferred: std::rc::Rc::new(RefCell::new(preferred)),
            fail_query: std::rc::Rc::new(RefCell::new(false)),
        }
    }

    fn set_failing(&self, fail: bool) {
        *self.fail_query.borrow_mut() = fail;
    }
}

impl CompositorAdapter for DynamicMockAdapter {
    fn query_output_facts(&self) -> Vec<CompositorOutputFacts> {
        if *self.fail_query.borrow() {
            Vec::new()
        } else {
            self.facts.borrow().clone()
        }
    }

    fn preferred_output_name(&self) -> Option<String> {
        if *self.fail_query.borrow() {
            None
        } else {
            self.preferred.borrow().clone()
        }
    }
}

#[test]
fn compositor_enrichment_duplicate_facts_for_same_connector_are_rejected() {
    let facts = vec![
        CompositorOutputFacts {
            name: "DP-1".into(),
            make: Some("Dell".into()),
            model: Some("U2720Q".into()),
            serial: Some("SN-1".into()),
            focused: true,
        },
        CompositorOutputFacts {
            name: "DP-1".into(),
            make: Some("Dell".into()),
            model: Some("DifferentModel".into()),
            serial: Some("SN-2".into()),
            focused: false,
        },
    ];

    let adapter = DynamicMockAdapter::new(facts, None);
    let mut mgr = OutputManager::new(Box::new(adapter));

    let proto = make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)));
    mgr.sync_protocol_facts(&[proto]);

    let snap = mgr.snapshot();
    let out = &snap.outputs()[0];
    // Ambiguous enrichment was rejected: make/model from protocol used, serial remains None
    assert_eq!(out.make(), "TestMake");
    assert_eq!(out.model(), "TestModel");
    assert_eq!(out.serial(), None);
    assert!(!out.preferred);
}

#[test]
fn compositor_stale_facts_from_removed_output_cannot_enrich_reannounced_output() {
    let adapter = DynamicMockAdapter::new(
        vec![CompositorOutputFacts {
            name: "DP-1".into(),
            make: Some("OldVendor".into()),
            model: Some("OldMonitor".into()),
            serial: Some("OLD-SN".into()),
            focused: false,
        }],
        None,
    );

    let mut mgr = OutputManager::new(Box::new(adapter.clone()));

    // 1. Initial output announced with compositor enrichment
    mgr.sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);
    let initial_id = mgr.snapshot().outputs()[0].id;
    assert_eq!(mgr.snapshot().outputs()[0].serial(), Some("OLD-SN"));

    // 2. Output is removed (monitor unplugged)
    mgr.sync_protocol_facts(&[]);
    assert!(mgr.snapshot().is_empty());

    // 3. Compositor query fails or facts are cleared when new monitor is plugged in
    adapter.set_failing(true);

    // 4. Re-announced on same connector with new native identity
    mgr.sync_protocol_facts(&[make_proto_output(2, "DP-1", (0, 0), Some((1920, 1080)))]);
    let new_snap = mgr.snapshot();
    assert_eq!(new_snap.len(), 1);
    let new_out = &new_snap.outputs()[0];
    assert_ne!(new_out.id, initial_id);
    assert_eq!(
        new_out.serial(),
        None,
        "stale serial must not attach to re-announced output"
    );
    assert_eq!(new_out.make(), "TestMake");
}

#[test]
fn compositor_query_failure_does_not_apply_stale_identity_to_new_output() {
    let dynamic_adapter = DynamicMockAdapter::new(
        vec![CompositorOutputFacts {
            name: "DP-1".into(),
            make: Some("CustomMake".into()),
            model: Some("CustomModel".into()),
            serial: Some("CUSTOM-SERIAL".into()),
            focused: true,
        }],
        Some("DP-1".into()),
    );

    dynamic_adapter.set_failing(true);
    let mut mgr = OutputManager::new(Box::new(dynamic_adapter));

    mgr.sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);
    let snap = mgr.snapshot();
    let out = &snap.outputs()[0];
    assert_eq!(out.make(), "TestMake");
    assert_eq!(out.serial(), None);
    assert!(!out.preferred);
}

#[test]
fn generic_output_remains_usable_without_enrichment() {
    let mut mgr = OutputManager::new(Box::new(GenericAdapter));
    mgr.sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);

    let snapshot = mgr.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot.outputs()[0].make(), "TestMake");
    assert_eq!(snapshot.outputs()[0].model(), "TestModel");
    assert_eq!(snapshot.outputs()[0].serial(), None);
    assert!(!snapshot.outputs()[0].preferred);

    let capable: Vec<_> = snapshot.placement_capable_outputs().collect();
    assert_eq!(capable.len(), 1);
}

// ---------------------------------------------------------------------------
// 3. TopologyGeneration Batching / Transaction Tests
// ---------------------------------------------------------------------------

#[test]
fn batch_refresh_advances_generation_exactly_once_per_committed_change() {
    let mut mgr = OutputManager::new(Box::new(GenericAdapter));
    assert_eq!(mgr.generation(), TopologyGeneration::new(0));

    // 1. One-output add refresh -> gen 1
    let commit1 =
        mgr.sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);
    assert!(commit1.is_some());
    assert_eq!(mgr.generation(), TopologyGeneration::new(1));
    assert_eq!(commit1.unwrap().generation(), TopologyGeneration::new(1));

    // 2. Multi-output add in one refresh -> gen 2 (advances exactly +1, not +2)
    let commit2 = mgr.sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080))),
        make_proto_output(2, "DP-2", (1920, 0), Some((2560, 1440))),
        make_proto_output(3, "HDMI-1", (4480, 0), Some((1920, 1080))),
    ]);
    assert!(commit2.is_some());
    assert_eq!(mgr.generation(), TopologyGeneration::new(2));
    assert_eq!(commit2.unwrap().generation(), TopologyGeneration::new(2));

    // 3. Multi-output update in one refresh -> gen 3 (advances exactly +1)
    let commit3 = mgr.sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080))),
        make_proto_output(2, "DP-2", (1920, 0), Some((3840, 2160))), // changed size
        make_proto_output(3, "HDMI-1", (5760, 0), Some((1920, 1080))), // changed pos
    ]);
    assert!(commit3.is_some());
    assert_eq!(mgr.generation(), TopologyGeneration::new(3));

    // 4. Identical refresh -> generation UNCHANGED, returns None
    let commit_same = mgr.sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080))),
        make_proto_output(2, "DP-2", (1920, 0), Some((3840, 2160))),
        make_proto_output(3, "HDMI-1", (5760, 0), Some((1920, 1080))),
    ]);
    assert!(commit_same.is_none());
    assert_eq!(mgr.generation(), TopologyGeneration::new(3));

    // 5. Add + update + remove all in one refresh -> gen 4 (advances exactly +1)
    let commit4 = mgr.sync_protocol_facts(&[
        make_proto_output(1, "DP-1", (0, 0), Some((2560, 1440))), // updated
        // DP-2 removed
        make_proto_output(3, "HDMI-1", (2560, 0), Some((1920, 1080))), // updated
        make_proto_output(4, "DP-3", (4480, 0), Some((1920, 1080))),   // added
    ]);
    assert!(commit4.is_some());
    assert_eq!(mgr.generation(), TopologyGeneration::new(4));
    let snap4 = commit4.unwrap();
    assert_eq!(snap4.len(), 3);
    assert!(snap4.find_by_name("DP-2").is_none());
    assert!(snap4.find_by_name("DP-3").is_some());
}

#[test]
fn protocol_update_and_enrichment_change_committed_together_advances_generation_by_one() {
    let mock = DynamicMockAdapter::new(
        vec![CompositorOutputFacts {
            name: "DP-1".into(),
            make: Some("Dell".into()),
            model: Some("U2720Q".into()),
            serial: Some("SN-1".into()),
            focused: true,
        }],
        Some("DP-1".into()),
    );

    let mut mgr = OutputManager::new(Box::new(mock));
    let commit =
        mgr.sync_protocol_facts(&[make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)))]);
    assert!(commit.is_some());
    assert_eq!(mgr.generation(), TopologyGeneration::new(1));
    let snap = commit.unwrap();
    assert_eq!(snap.generation(), TopologyGeneration::new(1));
    assert_eq!(snap.outputs()[0].serial(), Some("SN-1"));
    assert!(snap.outputs()[0].preferred);
}

// ---------------------------------------------------------------------------
// 4. Deterministic Canonical Snapshot Ordering Tests
// ---------------------------------------------------------------------------

#[test]
fn canonical_snapshot_ordering_is_deterministic_regardless_of_discovery_order() {
    let out_a = make_proto_output(1, "DP-1", (0, 0), Some((1920, 1080)));
    let out_b = make_proto_output(2, "DP-2", (1920, 0), Some((2560, 1440)));
    let out_c = make_proto_output(3, "HDMI-1", (4480, 0), Some((1920, 1080)));

    // Order 1: A, B, C
    let mut mgr1 = OutputManager::new(Box::new(GenericAdapter));
    mgr1.sync_protocol_facts(&[out_a.clone(), out_b.clone(), out_c.clone()]);
    let snap1 = mgr1.snapshot();

    // Order 2: C, A, B
    let mut mgr2 = OutputManager::new(Box::new(GenericAdapter));
    mgr2.sync_protocol_facts(&[out_c.clone(), out_a.clone(), out_b.clone()]);
    let snap2 = mgr2.snapshot();

    // Order 3: B, C, A
    let mut mgr3 = OutputManager::new(Box::new(GenericAdapter));
    mgr3.sync_protocol_facts(&[out_b, out_c, out_a]);
    let snap3 = mgr3.snapshot();

    let names1: Vec<_> = snap1.outputs().iter().map(|o| o.name.as_deref()).collect();
    let names2: Vec<_> = snap2.outputs().iter().map(|o| o.name.as_deref()).collect();
    let names3: Vec<_> = snap3.outputs().iter().map(|o| o.name.as_deref()).collect();

    assert_eq!(names1, vec![Some("DP-1"), Some("DP-2"), Some("HDMI-1")]);
    assert_eq!(
        names1, names2,
        "canonical ordering must be independent of discovery order"
    );
    assert_eq!(
        names1, names3,
        "canonical ordering must be independent of discovery order"
    );
}

// ---------------------------------------------------------------------------
// 5. Native Routing Lifecycle Tests
// ---------------------------------------------------------------------------

#[test]
fn native_routing_lifecycle_announce_remove_reannounce() {
    let mut mgr = OutputManager::new(Box::new(GenericAdapter));

    // 1. Announce native_id 100 on DP-1
    mgr.sync_protocol_facts(&[make_proto_output(100, "DP-1", (0, 0), Some((1920, 1080)))]);
    let id1 = mgr.find_id_by_native_id(100).expect("mapped ID");
    assert_eq!(id1, OutputId::new(1));
    assert!(mgr.snapshot().get(id1).is_some());

    // 2. Synchronously remove native_id 100
    let removed = mgr.remove_native_id(100);
    assert!(removed.is_some());
    let (rem_id, rem_snap, post_remove_topo) = removed.unwrap();
    assert_eq!(rem_id, id1);
    assert_eq!(rem_snap.name.as_deref(), Some("DP-1"));
    assert!(
        post_remove_topo.get(id1).is_none(),
        "absent from post-remove committed topology"
    );
    assert_eq!(mgr.find_id_by_native_id(100), None, "native route removed");

    // 3. Stale old native_id 100 cannot remove or update anything
    assert!(mgr.remove_native_id(100).is_none());

    // 4. Re-announce DP-1 with new native_id 101
    mgr.sync_protocol_facts(&[make_proto_output(101, "DP-1", (0, 0), Some((1920, 1080)))]);
    let id2 = mgr.find_id_by_native_id(101).expect("mapped new ID");
    assert_ne!(id2, id1, "fresh OutputId allocated on re-announcement");
    assert_eq!(id2, OutputId::new(2));
    assert_eq!(
        mgr.find_id_by_native_id(100),
        None,
        "stale native ID not revived"
    );
}

// ---------------------------------------------------------------------------
// 6. Surface & Token State Foundation Tests (from Step 1)
// ---------------------------------------------------------------------------

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
    assert_eq!(removed.map(|(o, _)| o.id), Some(id1));
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
fn surface_token_validation_rejects_wrong_host_id() {
    let mut registry = SurfaceGenerationRegistry::new();
    let host_a = HostId::new("host-a");
    let host_b = HostId::new("host-b");

    let token_a = registry.allocate(host_a.clone());
    let token_b = registry.allocate(host_b.clone());

    assert_eq!(token_a.generation, SurfaceGeneration::new(1));
    assert_eq!(token_b.generation, SurfaceGeneration::new(1));

    assert!(registry.validate(&host_a, &token_a));
    assert!(
        !registry.validate(&host_b, &token_a),
        "host-a token must NOT validate when checked for host-b"
    );

    assert!(registry.validate(&host_b, &token_b));
    assert!(
        !registry.validate(&host_a, &token_b),
        "host-b token must NOT validate when checked for host-a"
    );

    let token_a_gen2 = registry.recreate(&host_a);
    assert_eq!(token_a_gen2.generation, SurfaceGeneration::new(2));
    assert!(registry.validate(&host_a, &token_a_gen2));
    assert!(
        !registry.validate(&host_a, &token_a),
        "stale host-a generation 1 token must fail"
    );

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

    let token1 = registry.allocate(cal_id.clone());
    assert_eq!(token1.generation, SurfaceGeneration::new(1));
    assert!(registry.validate(&cal_id, &token1));

    let retired1 = registry.retire(&cal_id);
    assert_eq!(retired1, Some(token1.clone()));
    assert!(!registry.validate(&cal_id, &token1));
    assert_eq!(registry.current_token(&cal_id), None);
    assert_eq!(registry.current_generation(&cal_id), None);
    assert!(!registry.has_active_surface(&cal_id));

    let retired_again = registry.retire(&cal_id);
    assert_eq!(retired_again, None);

    let token2 = registry.allocate(cal_id.clone());
    assert_eq!(token2.generation, SurfaceGeneration::new(2));
    assert!(registry.validate(&cal_id, &token2));
    assert!(!registry.validate(&cal_id, &token1));

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

#[test]
fn surface_lifecycle_host_recreation_on_output_target_change() {
    let mut registry = SurfaceGenerationRegistry::new();
    let host_id = HostId::new("calendar");

    // 1. Initial surface on Output 1
    let token_output1 = registry.allocate(host_id.clone());
    assert_eq!(token_output1.generation().raw(), 1);
    assert!(registry.validate(&host_id, &token_output1));

    // 2. Output 1 is removed, triggering target change to fallback Output 2
    // Host invalidates old native surface by advancing generation
    let retired_token = registry.retire(&host_id);
    assert_eq!(retired_token, Some(token_output1.clone()));

    // Stale token from Output 1 is now rejected
    assert!(!registry.validate(&host_id, &token_output1));

    // 3. New surface created for fallback Output 2
    let token_output2 = registry.allocate(host_id.clone());
    assert_eq!(token_output2.generation().raw(), 2);
    assert!(registry.validate(&host_id, &token_output2));

    // Old token remains invalid
    assert!(!registry.validate(&host_id, &token_output1));
}

#[test]
fn surface_manager_token_validation_and_stale_callback_rejection() {
    let mut sm = SurfaceManager::new();
    let host_id = HostId::new("calendar");

    let token_gen1 = sm
        .generation_registry_mut_for_test()
        .allocate(host_id.clone());
    assert_eq!(token_gen1.generation.raw(), 1);
    assert!(sm.is_current(&token_gen1));

    // Invalidate gen1 by allocating gen2
    let token_gen2 = sm.generation_registry_mut_for_test().recreate(&host_id);
    assert_eq!(token_gen2.generation.raw(), 2);

    // gen2 is current, gen1 is stale and rejected
    assert!(sm.is_current(&token_gen2));
    assert!(!sm.is_current(&token_gen1));

    // Stale callback token validation fails
    assert!(!sm.generation_registry().validate(&host_id, &token_gen1));
    assert!(sm.generation_registry().validate(&host_id, &token_gen2));

    // Retiring leaves no active surface token
    let retired = sm.retire(&host_id);
    assert_eq!(retired, Some(token_gen2));
    assert_eq!(sm.current_token(&host_id), None);
    assert!(!sm.has_native_instance(&host_id));
}
