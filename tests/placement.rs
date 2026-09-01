use owlenshell::domain::ids::OutputId;
use owlenshell::domain::output::{OutputIdentity, OutputPreference, OutputSnapshot};
use owlenshell::domain::placement::{Anchor, ResolveReason, SavedPlacement};
use owlenshell::placement::{
    clamp_offset, containing_output, dragged_origin, margins_from_offset, nearest_corner,
    origin_from_margins, HostPlacementConstraints, PlacementContext, PlacementEngine, MIN_VISIBLE,
};

#[allow(clippy::too_many_arguments)]
fn snap(
    id: u32,
    name: &str,
    make: &str,
    model: &str,
    phys: (i32, i32),
    logical: Option<(i32, i32)>,
    serial: Option<&str>,
    preferred: bool,
) -> OutputSnapshot {
    OutputSnapshot {
        id: OutputId::new(id),
        name: Some(name.into()),
        identity: OutputIdentity {
            make: if make.is_empty() {
                None
            } else {
                Some(make.into())
            },
            model: if model.is_empty() {
                None
            } else {
                Some(model.into())
            },
            physical_size_mm: Some(phys),
            serial: serial.map(|s| s.into()),
        },
        logical_position: Some((0, 0)),
        logical_size: logical,
        logical_role: None,
        scale: 1,
        preferred,
    }
}

fn pref_name(name: &str) -> OutputPreference {
    OutputPreference::new().with_name(name)
}
fn test_resolve(
    pref: &OutputPreference,
    outputs: &[OutputSnapshot],
    ctx: &PlacementContext,
) -> owlenshell::domain::placement::PlacementDecision {
    let saved = SavedPlacement {
        output: pref.clone(),
        anchor: Anchor::TopRight,
        offset: (0, 0),
    };
    PlacementEngine::resolve(
        Some(&saved),
        outputs,
        ctx,
        &HostPlacementConstraints::default(),
    )
}

#[test]
fn single_monitor_exact_name() {
    let outs = [snap(
        1,
        "DP-1",
        "A",
        "M",
        (300, 200),
        Some((1920, 1080)),
        None,
        true,
    )];
    let r = test_resolve(&pref_name("DP-1"), &outs, &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::ExactName);
    assert_eq!(r.target_output_id(), Some(OutputId::new(1)));
    assert!(!r.is_fallback());
    assert!(!r.is_deferred());
    assert!(r.is_reachable());
}

#[test]
fn output_absent_falls_back() {
    let outs = [snap(
        2,
        "HDMI-A-1",
        "B",
        "N",
        (400, 250),
        Some((1920, 1080)),
        None,
        false,
    )];
    let r = test_resolve(&pref_name("DP-1"), &outs, &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::Any);
    assert_eq!(r.target_output_id(), Some(OutputId::new(2)));
    assert!(r.is_fallback(), "absent output must be marked as fallback");
}

#[test]
fn serial_survives_connector_rename() {
    let outs = [snap(
        9,
        "DP-3",
        "GB",
        "MO27Q28G",
        (600, 340),
        Some((2560, 1440)),
        Some("25392F001456"),
        false,
    )];
    let pref = OutputPreference::new()
        .with_name("DP-1")
        .with_serial("25392F001456")
        .with_make("GB")
        .with_model("MO27Q28G")
        .with_physical_size_mm((600, 340));
    let r = test_resolve(&pref, &outs, &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::Serial);
    assert_eq!(r.target_output_id(), Some(OutputId::new(9)));
    assert!(!r.is_fallback());
}

#[test]
fn unique_fingerprint_when_name_missing() {
    let outs = [
        snap(
            1,
            "DP-1",
            "Acer",
            "VG270",
            (600, 340),
            Some((2560, 1440)),
            None,
            false,
        ),
        snap(
            2,
            "HDMI-A-1",
            "LG",
            "FULL HD",
            (480, 270),
            Some((1920, 1080)),
            None,
            false,
        ),
    ];
    let pref = OutputPreference::new()
        .with_name("OLD-1")
        .with_make("Acer")
        .with_model("VG270")
        .with_physical_size_mm((600, 340));
    let r = test_resolve(&pref, &outs, &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::Fingerprint);
    assert_eq!(r.target_output_id(), Some(OutputId::new(1)));
    assert!(!r.is_fallback());
}

#[test]
fn ambiguous_fingerprint_skips() {
    let outs = [
        snap(
            1,
            "DP-1",
            "Dell",
            "U2720Q",
            (600, 340),
            Some((3840, 2160)),
            None,
            false,
        ),
        snap(
            2,
            "DP-2",
            "Dell",
            "U2720Q",
            (600, 340),
            Some((3840, 2160)),
            None,
            false,
        ),
    ];
    let pref = OutputPreference::new()
        .with_name("GONE")
        .with_make("Dell")
        .with_model("U2720Q")
        .with_physical_size_mm((600, 340));
    let r = test_resolve(
        &pref,
        &outs,
        &PlacementContext {
            spawn_output_id: Some(OutputId::new(2)),
            preferred_name: None,
            focused_output_id: None,
        },
    );
    assert_eq!(r.reason(), ResolveReason::SpawnOutput);
    assert_eq!(r.target_output_id(), Some(OutputId::new(2)));
    assert!(r.is_fallback());
}

#[test]
fn ambiguous_serial_skips() {
    let mut first = snap(
        1,
        "DP-1",
        "Vendor",
        "Panel",
        (600, 340),
        Some((1920, 1080)),
        Some("placeholder"),
        false,
    );
    let mut second = first.clone();
    second.id = OutputId::new(2);
    second.name = Some("DP-2".into());
    first.preferred = false;
    let pref = OutputPreference::new()
        .with_name("OLD")
        .with_serial("placeholder");
    let r = test_resolve(&pref, &[first, second], &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::Any);
    assert_eq!(r.target_output_id(), Some(OutputId::new(1)));
    assert!(r.is_fallback());
}

#[test]
fn logical_role_matching_is_honest() {
    let mut out1 = snap(
        1,
        "DP-1",
        "Vendor1",
        "Panel1",
        (600, 340),
        Some((1920, 1080)),
        None,
        false,
    );
    let mut out2 = snap(
        2,
        "DP-2",
        "Vendor2",
        "Panel2",
        (600, 340),
        Some((1920, 1080)),
        None,
        false,
    );
    out2.logical_role = Some("status-panel".into());
    out1.logical_role = Some("main-workspace".into());

    let pref = OutputPreference::new().with_logical_role("status-panel");
    let r = test_resolve(&pref, &[out1, out2], &PlacementContext::default());

    assert_eq!(r.reason(), ResolveReason::LogicalRole);
    assert_eq!(r.target_output_id(), Some(OutputId::new(2)));
    assert!(!r.is_fallback());

    // If role does not match, does NOT match connector name accidentally
    let pref_missing_role = OutputPreference::new().with_logical_role("non-existent-role");
    let r_missing = test_resolve(
        &pref_missing_role,
        &[snap(
            1,
            "non-existent-role",
            "A",
            "M",
            (1, 1),
            Some((1920, 1080)),
            None,
            false,
        )],
        &PlacementContext::default(),
    );
    // Should fall through to Any because logical_role did not match output's logical_role
    assert_eq!(r_missing.reason(), ResolveReason::Any);
}

#[test]
fn preferred_then_any() {
    let outs = [
        snap(
            5,
            "HDMI-A-1",
            "LG",
            "A",
            (400, 250),
            Some((1920, 1080)),
            None,
            false,
        ),
        snap(
            3,
            "DP-2",
            "Acer",
            "B",
            (500, 280),
            Some((2560, 1440)),
            None,
            true,
        ),
    ];
    let r = test_resolve(
        &OutputPreference::default(),
        &outs,
        &PlacementContext::default(),
    );
    assert_eq!(r.reason(), ResolveReason::Preferred);
    assert_eq!(r.target_output_id(), Some(OutputId::new(3)));
}

#[test]
fn deterministic_fallback_orders_by_stable_properties_ahead_of_runtime_id() {
    let outs = [
        snap(1, "HDMI-1", "B", "B", (1, 1), Some((800, 600)), None, false),
        snap(99, "DP-1", "A", "A", (1, 1), Some((800, 600)), None, false),
    ];
    let r = test_resolve(
        &OutputPreference::default(),
        &outs,
        &PlacementContext::default(),
    );
    assert_eq!(r.reason(), ResolveReason::Any);
    assert_eq!(
        r.target_output_id(),
        Some(OutputId::new(99)),
        "stable property (name DP-1) must order before HDMI-1 regardless of runtime OutputId"
    );
}

#[test]
fn no_outputs_defers() {
    let r = test_resolve(&pref_name("DP-1"), &[], &PlacementContext::default());
    assert_eq!(r.reason(), ResolveReason::Deferred);
    assert_eq!(r.target_output_id(), None);
    assert!(r.is_deferred());
    assert!(!r.is_reachable());
}

#[test]
fn fallback_does_not_change_home_resolution() {
    let home = snap(
        1,
        "DP-1",
        "A",
        "Home",
        (600, 340),
        Some((2560, 1440)),
        Some("home-serial"),
        false,
    );
    let fallback = snap(
        2,
        "HDMI-A-1",
        "B",
        "Fallback",
        (480, 270),
        Some((1920, 1080)),
        None,
        false,
    );
    let pref = OutputPreference::new()
        .with_name("DP-1")
        .with_serial("home-serial");

    let r_fallback = test_resolve(
        &pref,
        std::slice::from_ref(&fallback),
        &PlacementContext::default(),
    );
    assert_eq!(r_fallback.target_output_id(), Some(OutputId::new(2)));
    assert!(r_fallback.is_fallback());

    let r_home = test_resolve(&pref, &[fallback, home], &PlacementContext::default());
    assert_eq!(r_home.target_output_id(), Some(OutputId::new(1)));
    assert_eq!(r_home.reason(), ResolveReason::ExactName);
    assert!(!r_home.is_fallback());
}

#[test]
fn repeated_empty_and_restore_topologies_are_stable() {
    let output = snap(
        7,
        "DP-1",
        "A",
        "M",
        (600, 340),
        Some((2560, 1440)),
        None,
        false,
    );
    let pref = pref_name("DP-1");
    for outputs in [
        Vec::new(),
        vec![output.clone()],
        Vec::new(),
        vec![output.clone()],
    ] {
        let result = test_resolve(&pref, &outputs, &PlacementContext::default());
        assert_eq!(result.target_output_id(), outputs.first().map(|o| o.id));
    }
}

#[test]
fn removed_output_cannot_win_post_removal_resolution() {
    let removed = snap(
        1,
        "DP-1",
        "A",
        "Removed",
        (600, 340),
        Some((2560, 1440)),
        Some("removed-serial"),
        true,
    );
    let remaining = snap(
        2,
        "DP-2",
        "B",
        "Remaining",
        (600, 340),
        Some((2560, 1440)),
        None,
        false,
    );
    let pref = OutputPreference::new()
        .with_name("DP-1")
        .with_serial("removed-serial");
    let context = PlacementContext {
        spawn_output_id: Some(removed.id),
        preferred_name: removed.name.clone(),
        focused_output_id: None,
    };

    let result = test_resolve(&pref, std::slice::from_ref(&remaining), &context);

    assert_eq!(result.target_output_id(), Some(remaining.id));
    assert_eq!(result.reason(), ResolveReason::Any);
    assert!(result.is_fallback());
}

#[test]
fn clamp_oversized_offset() {
    let got = clamp_offset(
        Anchor::TopLeft,
        (5000, 4000),
        (320, 320),
        Some((1920, 1080)),
        None,
        MIN_VISIBLE,
    );
    assert_eq!(got, (1920 - 320, 1080 - 320));
}

#[test]
fn clamp_oversized_widget() {
    let got = clamp_offset(
        Anchor::TopLeft,
        (10, 10),
        (4000, 3000),
        Some((1920, 1080)),
        None,
        MIN_VISIBLE,
    );
    assert_eq!(got, (0, 0));
}

#[test]
fn clamp_tiny_widget_keeps_grabbable() {
    let got = clamp_offset(
        Anchor::TopLeft,
        (2000, 2000),
        (10, 10),
        Some((1920, 1080)),
        None,
        MIN_VISIBLE,
    );
    assert_eq!(got, (1920 - MIN_VISIBLE, 1080 - MIN_VISIBLE));
}

#[test]
fn clamp_anchor_top_left_with_non_zero_origin() {
    // Work area: top 32px and left 48px reserved
    let work_area = Some(((48, 32), (1872, 1048)));
    let got = clamp_offset(
        Anchor::TopLeft,
        (10, 5),
        (300, 200),
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    assert_eq!(
        got,
        (48, 32),
        "TopLeft anchor offset must clamp to non-zero work area origin (left=48, top=32)"
    );
}

#[test]
fn clamp_anchor_top_right_with_right_reserved_area() {
    // Output 1920x1080, Work area width 1820 (right 100px reserved for panel/dock)
    let work_area = Some(((0, 0), (1820, 1080)));
    let got = clamp_offset(
        Anchor::TopRight,
        (10, 40), // wants 10px from right edge
        (300, 200),
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    assert_eq!(
        got,
        (100, 40),
        "TopRight anchor offset must clamp to right-side reserved bound (right=100)"
    );
}

#[test]
fn clamp_anchor_bottom_left_with_bottom_reserved_area() {
    // Output 1920x1080, Work area height 1020 (bottom 60px reserved for bottom bar)
    let work_area = Some(((0, 0), (1920, 1020)));
    let got = clamp_offset(
        Anchor::BottomLeft,
        (50, 10), // wants 10px from bottom edge
        (300, 200),
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    assert_eq!(
        got,
        (50, 60),
        "BottomLeft anchor offset must clamp to bottom reserved bound (bottom=60)"
    );
}

#[test]
fn clamp_anchor_bottom_right_with_origin_and_reserved_bounds() {
    // Output 1920x1080. Work area: left 40, top 30, right 50, bottom 40
    // Origin (40, 30), Size (1830, 1010) -> right reserved = 1920 - (40+1830) = 50, bottom reserved = 1080 - (30+1010) = 40
    let work_area = Some(((40, 30), (1830, 1010)));
    let got = clamp_offset(
        Anchor::BottomRight,
        (10, 5), // wants 10px right, 5px bottom
        (300, 200),
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    assert_eq!(
        got,
        (50, 40),
        "BottomRight anchor offset must clamp to both right (50) and bottom (40) reserved bounds"
    );

    let got_max = clamp_offset(
        Anchor::BottomRight,
        (5000, 5000), // oversized offset
        (300, 200),
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    // Max span x = 1830 - 300 = 1530 -> max offset x = 50 + 1530 = 1580 (which leaves widget left edge at 1920 - 1580 - 300 = 40)
    // Max span y = 1010 - 200 = 810 -> max offset y = 40 + 810 = 850 (which leaves widget top edge at 1080 - 850 - 200 = 30)
    assert_eq!(
        got_max,
        (1580, 850),
        "BottomRight anchor offset must clamp to maximum usable work-area span"
    );
}

#[test]
fn clamp_oversized_widget_with_non_top_left_anchor() {
    // Output 1920x1080, Work area with right 50px and bottom 40px reserved
    let work_area = Some(((0, 0), (1870, 1040)));
    let got = clamp_offset(
        Anchor::BottomRight,
        (10, 10),
        (3000, 2000), // oversized widget
        Some((1920, 1080)),
        work_area,
        MIN_VISIBLE,
    );
    assert_eq!(
        got,
        (50, 40),
        "Oversized widget with BottomRight anchor must clamp to reserved boundary (50, 40) to stay grabbable"
    );
}

#[test]
fn clamp_skipped_without_logical_size() {
    let got = clamp_offset(
        Anchor::TopLeft,
        (999, 999),
        (320, 320),
        None,
        None,
        MIN_VISIBLE,
    );
    assert_eq!(got, (999, 999));
}

#[test]
fn nearest_corner_picks_top_left() {
    let (anchor, offset) = nearest_corner((10, 12), (320, 320), (1920, 1080));
    assert_eq!(anchor, Anchor::TopLeft);
    assert_eq!(offset, (10, 12));
}

#[test]
fn nearest_corner_picks_bottom_right() {
    let (anchor, offset) = nearest_corner((1600, 800), (320, 320), (1920, 1080));
    assert_eq!(anchor, Anchor::BottomRight);
    assert_eq!(offset, (0, -40));
}

#[test]
fn overlapping_logical_outputs_are_ambiguous() {
    let outputs = [
        snap(1, "DP-1", "A", "A", (1, 1), Some((800, 600)), None, false),
        snap(2, "DP-2", "B", "B", (1, 1), Some((800, 600)), None, false),
    ];
    assert!(containing_output((100, 100), &outputs).is_none());
}

#[test]
fn release_drag_uses_delta_from_fixed_surface() {
    assert_eq!(
        dragged_origin((100, 80), (20.0, 10.0), (75.0, 42.0)),
        (155, 112)
    );
}

#[test]
fn semantic_placement_after_committed_move() {
    let mut out1 = snap(
        1,
        "DP-1",
        "Dell",
        "U2720Q",
        (600, 340),
        Some((1920, 1080)),
        Some("SN1"),
        true,
    );
    out1.logical_position = Some((0, 0));

    let mut out2 = snap(
        2,
        "DP-2",
        "LG",
        "4K",
        (600, 340),
        Some((1920, 1080)),
        Some("SN2"),
        false,
    );
    out2.logical_position = Some((1920, 0));

    let outputs = [out1, out2];

    // Drag within output 1: start at top-right, drag toward bottom-left
    let committed = PlacementEngine::commit_move(
        (50.0, 50.0),   // pointer
        (10.0, 10.0),   // grab
        (300, 400),     // widget size
        (20, 30, 0, 0), // start margins (TopRight: top 20, right 30)
        Anchor::TopRight,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("committed move should succeed");

    assert_eq!(committed.target_output_id, OutputId::new(1));
    assert_eq!(
        committed.saved_placement.output.name.as_deref(),
        Some("DP-1")
    );
    assert_eq!(committed.saved_placement.output.serial(), Some("SN1"));
    assert_eq!(committed.saved_placement.output.make(), Some("Dell"));
    assert_eq!(committed.saved_placement.output.model(), Some("U2720Q"));
    assert_eq!(
        committed.saved_placement.output.physical_size_mm(),
        Some((600, 340))
    );
    assert_eq!(committed.saved_placement.anchor, Anchor::TopRight);
    assert_eq!(committed.saved_placement.offset, (0, 60));

    // Cross-output move: pointer at global coordinate (2000, 100) on output 2
    let cross_move = PlacementEngine::commit_move(
        (450.0, 50.0), // pointer moving across boundary onto output 2
        (10.0, 10.0),
        (300, 400),
        (20, 30, 0, 0),
        Anchor::TopRight,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("cross output move must resolve target output 2");

    assert_eq!(
        cross_move.target_output_id,
        OutputId::new(2),
        "target output must specifically be OutputId(2)"
    );
    assert_eq!(
        cross_move.saved_placement.output.name.as_deref(),
        Some("DP-2")
    );
    assert_eq!(cross_move.saved_placement.output.serial(), Some("SN2"));
    assert_eq!(cross_move.saved_placement.output.make(), Some("LG"));
    assert_eq!(cross_move.saved_placement.output.model(), Some("4K"));
}

#[test]
fn cross_output_drop_vertically_and_horizontally_offset_outputs() {
    // Source output at (0, 1440), target output at (1920, 0)
    let mut out1 = snap(
        1,
        "DP-1",
        "Gigabyte",
        "MO27Q",
        (590, 330),
        Some((1920, 1080)),
        Some("SN1"),
        true,
    );
    out1.logical_position = Some((0, 1440));

    let mut out2 = snap(
        2,
        "DP-2",
        "Acer",
        "VG270U",
        (600, 340),
        Some((2560, 1440)),
        Some("SN2"),
        false,
    );
    out2.logical_position = Some((1920, 0));

    let outputs = [out1, out2];

    // Calendar widget on out1 (0, 1440): start anchored TopRight with offset (24, 40)
    let widget_size = (320, 320);
    let start_anchor = Anchor::TopRight;
    let start_offset = (24, 40);
    let start_margins = margins_from_offset(start_anchor, start_offset); // (40, 24, 0, 0)
    let grab = (10.0, 10.0);

    // Initial widget origin on out1: (1920 - 320 - 24, 40) = (1576, 40)
    // Global initial widget top-left: (0 + 1576, 1440 + 40) = (1576, 1480)

    // User drags pointer to desktop global (2500, 300) on out2 (1920, 0)
    // Wayland event position relative to source surface: (2500 - 1576, 300 - 1480) = (924.0, -1180.0)
    let pointer = (924.0, -1180.0);

    let committed = PlacementEngine::commit_move(
        pointer,
        grab,
        widget_size,
        start_margins,
        start_anchor,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("cross-output move onto out2 must succeed");

    // 1. Correct target output selected
    assert_eq!(committed.target_output_id, OutputId::new(2));
    assert_eq!(
        committed.saved_placement.output.name.as_deref(),
        Some("DP-2")
    );

    // 2. Expected target-local geometry:
    // global drop (2500, 300) - target_out.logical_position (1920, 0) - grab (10, 10) = local (570, 290)
    // Center: (570 + 160, 290 + 160) = (730, 450) on out2 (2560x1440) -> nearest corner is TopLeft
    assert_eq!(committed.saved_placement.anchor, Anchor::TopLeft);
    assert_eq!(committed.saved_placement.offset, (570, 290));

    // 3. Converting semantic placement back to target geometry reproduces intended drop location
    let target_margins = margins_from_offset(
        committed.saved_placement.anchor,
        committed.saved_placement.offset,
    );
    let target_origin = origin_from_margins(
        committed.saved_placement.anchor,
        target_margins,
        widget_size,
        (2560, 1440),
    );
    assert_eq!(target_origin, (570, 290));

    let target_global_top_left = (1920 + target_origin.0, target_origin.1);
    assert_eq!(target_global_top_left, (2490, 290));
    assert_eq!(
        (
            target_global_top_left.0 + grab.0 as i32,
            target_global_top_left.1 + grab.1 as i32
        ),
        (2500, 300),
        "reconstructed global pointer position must match original drop position"
    );
}

#[test]
fn cross_output_drop_left_to_right_and_right_to_left() {
    let mut out_left = snap(
        1,
        "DP-1",
        "Make1",
        "Model1",
        (600, 340),
        Some((1920, 1080)),
        Some("SN1"),
        true,
    );
    out_left.logical_position = Some((0, 0));

    let mut out_right = snap(
        2,
        "DP-2",
        "Make2",
        "Model2",
        (600, 340),
        Some((1920, 1080)),
        Some("SN2"),
        false,
    );
    out_right.logical_position = Some((1920, 0));

    let outputs = [out_left, out_right];
    let widget_size = (320, 320);
    let grab = (20.0, 20.0);

    // Left -> Right: start on out_left TopLeft (50, 50), drop on out_right at global (2500, 400)
    let start_margins = margins_from_offset(Anchor::TopLeft, (50, 50));
    // Pointer relative to surface: (2500 - 50, 400 - 50) = (2450, 350)
    let commit_r = PlacementEngine::commit_move(
        (2450.0, 350.0),
        grab,
        widget_size,
        start_margins,
        Anchor::TopLeft,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("left to right move");

    assert_eq!(commit_r.target_output_id, OutputId::new(2));
    // Local top-left on out_right: (2500 - 1920 - 20, 400 - 0 - 20) = (560, 380)
    // Center: (720, 540) on (1920, 1080) -> TopLeft
    assert_eq!(commit_r.saved_placement.anchor, Anchor::TopLeft);
    assert_eq!(commit_r.saved_placement.offset, (560, 380));

    // Right -> Left: start on out_right TopRight (30, 40), drop on out_left at global (500, 500)
    // Origin on out_right: (1920 - 320 - 30, 40) = (1570, 40)
    // Global start top-left: (1920 + 1570, 0 + 40) = (3490, 40)
    let start_margins_r = margins_from_offset(Anchor::TopRight, (30, 40));
    // Pointer relative to surface: (500 - 3490, 500 - 40) = (-2990, 460)
    let commit_l = PlacementEngine::commit_move(
        (-2990.0, 460.0),
        grab,
        widget_size,
        start_margins_r,
        Anchor::TopRight,
        OutputId::new(2),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("right to left move");

    assert_eq!(commit_l.target_output_id, OutputId::new(1));
    // Local top-left on out_left: (500 - 0 - 20, 500 - 0 - 20) = (480, 480)
    // Center: (640, 640) on (1920, 1080): distance to BottomLeft is 640^2 + 440^2 = 603200 vs TopLeft 819200
    // Nearest corner is BottomLeft with offset (480, 1080 - 480 - 320) = (480, 280)
    assert_eq!(commit_l.saved_placement.anchor, Anchor::BottomLeft);
    assert_eq!(commit_l.saved_placement.offset, (480, 280));

    // Converting back reproduces exact local and global drop location:
    let margins_l = margins_from_offset(
        commit_l.saved_placement.anchor,
        commit_l.saved_placement.offset,
    );
    let origin_l = origin_from_margins(
        commit_l.saved_placement.anchor,
        margins_l,
        widget_size,
        (1920, 1080),
    );
    assert_eq!(origin_l, (480, 480));
    assert_eq!(
        (origin_l.0 + grab.0 as i32, origin_l.1 + grab.1 as i32),
        (500, 500)
    );
}

#[test]
fn cross_output_drop_target_with_non_zero_x_and_y() {
    let mut out1 = snap(
        1,
        "DP-1",
        "Make1",
        "Model1",
        (600, 340),
        Some((1920, 1080)),
        None,
        false,
    );
    out1.logical_position = Some((0, 0));

    let mut out2 = snap(
        2,
        "DP-2",
        "Make2",
        "Model2",
        (600, 340),
        Some((2560, 1440)),
        None,
        false,
    );
    out2.logical_position = Some((2560, 1440));

    let outputs = [out1, out2];
    let widget_size = (300, 300);
    let grab = (0.0, 0.0);

    // Start on out1 at (0, 0), drop on out2 at global (3000, 2000)
    let start_margins = margins_from_offset(Anchor::TopLeft, (0, 0));
    let commit = PlacementEngine::commit_move(
        (3000.0, 2000.0),
        grab,
        widget_size,
        start_margins,
        Anchor::TopLeft,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("move onto diagonally offset output");

    assert_eq!(commit.target_output_id, OutputId::new(2));
    // Local on out2: (3000 - 2560 - 0, 2000 - 1440 - 0) = (440, 560)
    // Center: (590, 710) on (2560, 1440) -> TopLeft
    assert_eq!(commit.saved_placement.anchor, Anchor::TopLeft);
    assert_eq!(commit.saved_placement.offset, (440, 560));
}

#[test]
fn same_output_drop_preserves_exact_geometry() {
    let mut out = snap(
        1,
        "DP-1",
        "Make1",
        "Model1",
        (600, 340),
        Some((2560, 1440)),
        None,
        true,
    );
    out.logical_position = Some((1920, 1440));

    let outputs = [out];
    let widget_size = (320, 320);
    let grab = (15.0, 15.0);

    // Start at TopRight (24, 40): origin = (2560 - 320 - 24, 40) = (2216, 40)
    let start_margins = margins_from_offset(Anchor::TopRight, (24, 40));
    // User moves pointer by (+100, +200) relative to grab: pointer = (15 + 100, 15 + 200) = (115, 215)
    let commit = PlacementEngine::commit_move(
        (115.0, 215.0),
        grab,
        widget_size,
        start_margins,
        Anchor::TopRight,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("same output move");

    assert_eq!(commit.target_output_id, OutputId::new(1));
    // Expected local origin: (2216 + 100, 40 + 200) = (2316, 240)
    // Center: (2316 + 160, 240 + 160) = (2476, 400) on (2560, 1440) -> TopRight
    // Offset TopRight: (2560 - 2316 - 320, 240) = (-76 -> clamped to 0, 240)
    assert_eq!(commit.saved_placement.anchor, Anchor::TopRight);
    assert_eq!(commit.saved_placement.offset, (0, 240));
}

#[test]
fn runtime_try_attach_geometry_computation_uses_dynamic_preferred_size() {
    use owlenshell::runtime::{compute_attach_geometry, SurfaceKind};
    use slint::LogicalSize;

    // Preferred size below 320: 240x180
    let preferred = LogicalSize::new(240.0, 180.0);

    // On an output of size 400x300, offset (150, 50) with Anchor::TopRight:
    // With 240x180: max_span_x = 400 - 240 = 160, max_span_y = 300 - 180 = 120.
    // Offset (150, 50) fits within [0, 160] and [0, 120], so clamped offset is (150, 50).
    //
    // Under the old bug (hardcoded 320x320 clamp):
    // max_span_x = 400 - 320 = 80, max_span_y = 300 - 320 (clamped to 0) = 0.
    // The old buggy logic clamped offset (150, 50) to (80, 0), shifting X by 70px and Y by 50px!
    let placement = SavedPlacement {
        output: OutputPreference::new().with_name("DP-1"),
        anchor: Anchor::TopRight,
        offset: (150, 50),
    };

    let geo = compute_attach_geometry(
        SurfaceKind::Calendar,
        Some(&placement),
        preferred,
        Some((400, 300)),
    );

    assert_eq!(
        geo.effective_widget_size,
        (240, 180),
        "effective widget size must be derived from preferred size (240x180), not hardcoded 320x320"
    );
    assert_eq!(
        geo.requested_size,
        (240, 180),
        "requested native size must equal preferred size (240x180)"
    );
    assert_eq!(
        geo.offset,
        (150, 50),
        "offset (150, 50) must not be clamped to (80, 0) as would happen under a 320x320 assumption"
    );
    assert_eq!(
        geo.margins,
        (50, 150, 0, 0),
        "margins must match the (150, 50) offset, not the buggy (0, 80, 0, 0)"
    );
}

#[test]
fn dynamic_preferred_size_below_320_recreation_consistency() {
    let mut out1 = snap(
        1,
        "DP-1",
        "Make1",
        "Model1",
        (600, 340),
        Some((1920, 1080)),
        Some("SN1"),
        true,
    );
    out1.logical_position = Some((0, 0));

    let mut out2 = snap(
        2,
        "DP-2",
        "Make2",
        "Model2",
        (600, 340),
        Some((2560, 1440)),
        Some("SN2"),
        false,
    );
    out2.logical_position = Some((1920, 0));

    let outputs = [out1, out2];

    // Preferred widget size below 320 in both dimensions: 240x180
    let widget_size = (240, 180);
    let start_anchor = Anchor::TopRight;
    let start_offset = (20, 30);
    let start_margins = margins_from_offset(start_anchor, start_offset); // (30, 20, 0, 0)
    let grab = (10.0, 10.0);

    // Initial widget origin on out1: (1920 - 240 - 20, 30) = (1660, 30)
    let start_origin = origin_from_margins(start_anchor, start_margins, widget_size, (1920, 1080));
    assert_eq!(start_origin, (1660, 30));

    // 1. Move within out1 to target-local (1500, 50)
    // Pointer event relative to surface: (1500 - 1660 + 10, 50 - 30 + 10) = (-150.0, 30.0)
    let commit_same = PlacementEngine::commit_move(
        (-150.0, 30.0),
        grab,
        widget_size,
        start_margins,
        start_anchor,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("same output move with 240x180");

    assert_eq!(commit_same.target_output_id, OutputId::new(1));
    assert_eq!(commit_same.saved_placement.anchor, Anchor::TopRight);
    // With 240x180: offset.0 = 1920 - 1500 - 240 = 180.
    // If it had clamped with 320x320, offset.0 would have shifted to 100.
    assert_eq!(
        commit_same.saved_placement.offset,
        (180, 50),
        "offset must be computed using actual 240x180 size without 320 clamp shift"
    );

    let margins_same = margins_from_offset(
        commit_same.saved_placement.anchor,
        commit_same.saved_placement.offset,
    );
    let origin_same = origin_from_margins(
        commit_same.saved_placement.anchor,
        margins_same,
        widget_size,
        (1920, 1080),
    );
    assert_eq!(origin_same, (1500, 50));

    // 2. Cross-output move onto out2 at global (2400, 200)
    // Global start top-left: (0 + 1660, 0 + 30) = (1660, 30)
    // Pointer relative to surface: (2400 - 1660, 200 - 30) = (740.0, 170.0)
    let commit_cross = PlacementEngine::commit_move(
        (740.0, 170.0),
        grab,
        widget_size,
        start_margins,
        start_anchor,
        OutputId::new(1),
        &outputs,
        None,
        MIN_VISIBLE,
    )
    .expect("cross output move onto out2 with 240x180");

    assert_eq!(commit_cross.target_output_id, OutputId::new(2));
    // Local top-left on out2: (2400 - 1920 - 10, 200 - 0 - 10) = (470, 190)
    // Center: (470 + 120, 190 + 90) = (590, 280) on (2560, 1440) -> TopLeft
    assert_eq!(commit_cross.saved_placement.anchor, Anchor::TopLeft);
    assert_eq!(commit_cross.saved_placement.offset, (470, 190));

    // Clamping with 240x180 preserves exact (470, 190)
    let clamped_offset = clamp_offset(
        commit_cross.saved_placement.anchor,
        commit_cross.saved_placement.offset,
        widget_size,
        Some((2560, 1440)),
        None,
        MIN_VISIBLE,
    );
    assert_eq!(clamped_offset, (470, 190));

    let margins_cross = margins_from_offset(commit_cross.saved_placement.anchor, clamped_offset);
    let origin_cross = origin_from_margins(
        commit_cross.saved_placement.anchor,
        margins_cross,
        widget_size,
        (2560, 1440),
    );
    assert_eq!(origin_cross, (470, 190));
}

#[test]
fn pure_placement_engine_records_diagnostics_and_margins() {
    let out = snap(
        1,
        "DP-1",
        "Make",
        "Model",
        (600, 340),
        Some((1920, 1080)),
        None,
        false,
    );
    let saved = SavedPlacement {
        output: OutputPreference::new().with_name("DP-1"),
        anchor: Anchor::BottomLeft,
        offset: (50, 60),
    };
    let constraints = HostPlacementConstraints {
        min_visible: 32,
        widget_size: (300, 200),
        work_area: None,
    };

    let decision = PlacementEngine::resolve(
        Some(&saved),
        std::slice::from_ref(&out),
        &PlacementContext::default(),
        &constraints,
    );

    assert_eq!(decision.target_output_id(), Some(OutputId::new(1)));
    assert_eq!(decision.reason(), ResolveReason::ExactName);
    assert_eq!(decision.anchor(), Anchor::BottomLeft);
    assert_eq!(decision.offset(), (50, 60));
    assert_eq!(decision.margins(), (0, 0, 60, 50));
    assert!(!decision.is_fallback());
    assert!(!decision.is_deferred());
    assert!(decision.is_reachable());
}
