use owlenbar::model::{Anchor, OutputPreference, OutputSnapshot, MIN_VISIBLE};
use owlenbar::placement::{
    clamp_offset, containing_output, dragged_origin, nearest_corner, resolve, ResolveContext,
    ResolveReason,
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
        id,
        name: Some(name.into()),
        make: make.into(),
        model: model.into(),
        physical_size_mm: phys,
        logical_position: Some((0, 0)),
        logical_size: logical,
        scale: 1,
        serial: serial.map(|s| s.into()),
        preferred,
    }
}

fn pref_name(name: &str) -> OutputPreference {
    OutputPreference {
        name: Some(name.into()),
        ..OutputPreference::default()
    }
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
    let r = resolve(&pref_name("DP-1"), &outs, &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::ExactName);
    assert_eq!(r.output_id, Some(1));
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
    let r = resolve(&pref_name("DP-1"), &outs, &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::Any);
    assert_eq!(r.output_id, Some(2));
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
    let pref = OutputPreference {
        name: Some("DP-1".into()),
        serial: Some("25392F001456".into()),
        make: Some("GB".into()),
        model: Some("MO27Q28G".into()),
        physical_size_mm: Some((600, 340)),
    };
    let r = resolve(&pref, &outs, &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::Serial);
    assert_eq!(r.output_id, Some(9));
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
    let pref = OutputPreference {
        name: Some("OLD-1".into()),
        make: Some("Acer".into()),
        model: Some("VG270".into()),
        physical_size_mm: Some((600, 340)),
        serial: None,
    };
    let r = resolve(&pref, &outs, &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::Fingerprint);
    assert_eq!(r.output_id, Some(1));
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
    let pref = OutputPreference {
        name: Some("GONE".into()),
        make: Some("Dell".into()),
        model: Some("U2720Q".into()),
        physical_size_mm: Some((600, 340)),
        serial: None,
    };
    let r = resolve(
        &pref,
        &outs,
        &ResolveContext {
            spawn_output_id: Some(2),
            preferred_name: None,
        },
    );
    assert_eq!(r.reason, ResolveReason::SpawnOutput);
    assert_eq!(r.output_id, Some(2));
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
    second.id = 2;
    second.name = Some("DP-2".into());
    first.preferred = false;
    let pref = OutputPreference {
        name: Some("OLD".into()),
        serial: Some("placeholder".into()),
        ..OutputPreference::default()
    };
    let r = resolve(&pref, &[first, second], &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::Any);
    assert_eq!(r.output_id, Some(1));
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
    let r = resolve(
        &OutputPreference::default(),
        &outs,
        &ResolveContext::default(),
    );
    assert_eq!(r.reason, ResolveReason::Preferred);
    assert_eq!(r.output_id, Some(3));
}

#[test]
fn any_picks_minimum_id() {
    let outs = [
        snap(8, "DP-2", "A", "A", (1, 1), Some((800, 600)), None, false),
        snap(4, "DP-1", "B", "B", (1, 1), Some((800, 600)), None, false),
    ];
    let r = resolve(
        &OutputPreference::default(),
        &outs,
        &ResolveContext::default(),
    );
    assert_eq!(r.reason, ResolveReason::Any);
    assert_eq!(r.output_id, Some(4));
}

#[test]
fn no_outputs_defers() {
    let r = resolve(&pref_name("DP-1"), &[], &ResolveContext::default());
    assert_eq!(r.reason, ResolveReason::Deferred);
    assert_eq!(r.output_id, None);
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
    let pref = OutputPreference {
        name: Some("DP-1".into()),
        serial: Some("home-serial".into()),
        ..OutputPreference::default()
    };

    assert_eq!(
        resolve(
            &pref,
            std::slice::from_ref(&fallback),
            &ResolveContext::default()
        )
        .output_id,
        Some(2)
    );
    assert_eq!(
        resolve(&pref, &[fallback, home], &ResolveContext::default()).output_id,
        Some(1)
    );
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
        let result = resolve(&pref, &outputs, &ResolveContext::default());
        assert_eq!(result.output_id, outputs.first().map(|o| o.id));
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
    let pref = OutputPreference {
        name: removed.name.clone(),
        serial: removed.serial.clone(),
        ..OutputPreference::default()
    };
    let context = ResolveContext {
        spawn_output_id: Some(removed.id),
        preferred_name: removed.name.clone(),
    };

    let result = resolve(&pref, std::slice::from_ref(&remaining), &context);

    assert_eq!(result.output_id, Some(remaining.id));
    assert_eq!(result.reason, ResolveReason::Any);
}

#[test]
fn clamp_oversized_offset() {
    let got = clamp_offset((5000, 4000), (320, 320), Some((1920, 1080)));
    assert_eq!(got, (1920 - 320, 1080 - 320));
}

#[test]
fn clamp_oversized_widget() {
    let got = clamp_offset((10, 10), (4000, 3000), Some((1920, 1080)));
    assert_eq!(got, (0, 0));
}

#[test]
fn clamp_tiny_widget_keeps_grabbable() {
    let got = clamp_offset((2000, 2000), (10, 10), Some((1920, 1080)));
    assert_eq!(got, (1920 - MIN_VISIBLE, 1080 - MIN_VISIBLE));
}

#[test]
fn clamp_skipped_without_logical_size() {
    let got = clamp_offset((999, 999), (320, 320), None);
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
