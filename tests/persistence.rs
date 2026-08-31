use owlenshell::model::{Anchor, OutputPreference, SavedPlacement};
use owlenshell::persistence::Placements;
use std::path::PathBuf;

fn test_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "owlenshell-{name}-{}-{}.toml",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ))
}

#[test]
fn placement_round_trip() {
    let path = test_path("round-trip");
    let mut placements = Placements::default();
    placements.set(
        "calendar",
        SavedPlacement {
            output: OutputPreference {
                name: Some("DP-1".into()),
                serial: Some("serial".into()),
                ..OutputPreference::default()
            },
            anchor: Anchor::BottomRight,
            offset: (12, 34),
        },
    );

    placements.save_to(&path).unwrap();
    assert_eq!(
        Placements::load_from(&path).get("calendar"),
        placements.get("calendar")
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn corrupt_file_falls_back_to_defaults() {
    let path = test_path("corrupt");
    std::fs::write(&path, "not = [valid").unwrap();
    let loaded = Placements::load_from(&path);
    assert_eq!(loaded.version, 1);
    assert!(loaded.widgets.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn unsupported_version_falls_back_to_defaults() {
    let path = test_path("version");
    std::fs::write(&path, "version = 99\n[widgets]\n").unwrap();
    let loaded = Placements::load_from(&path);
    assert_eq!(loaded.version, 1);
    assert!(loaded.widgets.is_empty());
    let _ = std::fs::remove_file(path);
}
