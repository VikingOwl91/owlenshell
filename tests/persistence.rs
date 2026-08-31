use owlenshell::domain::ids::HostId;
use owlenshell::domain::output::OutputPreference;
use owlenshell::domain::placement::{Anchor, SavedPlacement};
use owlenshell::persistence::repository::LoadOutcome;
use owlenshell::persistence::schema::{AnchorDto, SavedPlacementDto, CURRENT_SCHEMA_VERSION};
use owlenshell::persistence::Placements;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
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
    let host_id = HostId::new("calendar");
    let saved = SavedPlacement {
        output: OutputPreference::new()
            .with_name("DP-1")
            .with_serial("serial-12345")
            .with_make("Dell")
            .with_model("U2720Q")
            .with_physical_size_mm((600, 340)),
        anchor: Anchor::BottomRight,
        offset: (12, 34),
    };
    placements.set(host_id.clone(), saved.clone());

    placements.save_to(&path).unwrap();
    let loaded = Placements::load_from(&path);
    assert_eq!(loaded.get(&host_id), Some(&saved));
    assert_eq!(loaded.version(), CURRENT_SCHEMA_VERSION);
    let _ = std::fs::remove_file(path);
}

#[test]
fn corrupt_file_preserves_backup_and_falls_back() {
    let path = test_path("corrupt-backup");
    let corrupt_content = "this is not = [valid toml content \n {{[[";
    std::fs::write(&path, corrupt_content).unwrap();

    let outcome = Placements::load_with_outcome(&path);
    match outcome {
        LoadOutcome::CorruptedPreserved {
            backup_path,
            error,
            fallback,
        } => {
            assert!(backup_path.exists(), "corrupt backup file must exist");
            assert!(
                backup_path.to_string_lossy().ends_with(".bak"),
                "backup extension must be .bak"
            );
            assert_eq!(
                std::fs::read_to_string(&backup_path).unwrap(),
                corrupt_content
            );
            assert!(!error.is_empty());
            assert_eq!(fallback.version(), CURRENT_SCHEMA_VERSION);
            assert!(fallback.is_empty());
            let _ = std::fs::remove_file(backup_path);
        }
        other => panic!("expected CorruptedPreserved, got: {:?}", other),
    }

    let loaded = Placements::load_from(&path);
    assert_eq!(loaded.version(), CURRENT_SCHEMA_VERSION);
    assert!(loaded.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn failed_preservation_reported_honestly() {
    let temp_dir = std::env::temp_dir().join(format!(
        "owlenshell-readonly-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let file_path = temp_dir.join("placements.toml");
    std::fs::write(&file_path, "invalid = [toml").unwrap();

    // Make directory read-only so backup creation fails
    let mut perms = std::fs::metadata(&temp_dir).unwrap().permissions();
    perms.set_mode(0o555);
    let _ = std::fs::set_permissions(&temp_dir, perms);

    let outcome = Placements::load_with_outcome(&file_path);

    // Restore permissions for cleanup
    let mut restore_perms = std::fs::metadata(&temp_dir).unwrap().permissions();
    restore_perms.set_mode(0o755);
    let _ = std::fs::set_permissions(&temp_dir, restore_perms);
    let _ = std::fs::remove_dir_all(&temp_dir);

    match outcome {
        LoadOutcome::PreservationFailed {
            backup_path: _,
            preserve_error,
            parse_error,
            fallback,
        } => {
            assert!(!preserve_error.is_empty());
            assert!(!parse_error.is_empty());
            assert!(fallback.is_empty());
        }
        other => panic!("expected PreservationFailed, got: {:?}", other),
    }
}

#[test]
fn unreadable_file_reports_read_failed() {
    let path = test_path("unreadable");
    File::create(&path).unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o000);
    let _ = std::fs::set_permissions(&path, perms);

    let outcome = Placements::load_with_outcome(&path);

    let mut restore = std::fs::metadata(&path).unwrap().permissions();
    restore.set_mode(0o644);
    let _ = std::fs::set_permissions(&path, restore);
    let _ = std::fs::remove_file(&path);

    match outcome {
        LoadOutcome::ReadFailed { error, fallback } => {
            assert!(!error.is_empty());
            assert!(fallback.is_empty());
        }
        other => panic!("expected ReadFailed, got: {:?}", other),
    }
}

#[test]
fn unsupported_version_falls_back_to_defaults() {
    let path = test_path("unsupported-version");
    std::fs::write(&path, "version = 99\n[widgets]\n").unwrap();

    let outcome = Placements::load_with_outcome(&path);
    match outcome {
        LoadOutcome::UnsupportedVersion { version, fallback } => {
            assert_eq!(version, 99);
            assert_eq!(fallback.version(), CURRENT_SCHEMA_VERSION);
            assert!(fallback.is_empty());
        }
        other => panic!("expected UnsupportedVersion, got: {:?}", other),
    }

    let loaded = Placements::load_from(&path);
    assert_eq!(loaded.version(), CURRENT_SCHEMA_VERSION);
    assert!(loaded.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn missing_file_returns_clean_default() {
    let path = test_path("nonexistent-file-xyz");
    let outcome = Placements::load_with_outcome(&path);
    assert_eq!(outcome, LoadOutcome::DefaultEmpty);
    let loaded = Placements::load_from(&path);
    assert_eq!(loaded, Placements::default());
}

#[test]
fn atomic_replacement_success_and_preservation_on_failure() {
    let test_dir = std::env::temp_dir().join(format!(
        "owlenshell-atomic-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let path = test_dir.join("placements.toml");

    let mut p1 = Placements::default();
    p1.set(
        HostId::new("host-1"),
        SavedPlacement {
            output: OutputPreference::new().with_name("DP-1"),
            anchor: Anchor::TopLeft,
            offset: (10, 20),
        },
    );
    p1.save_to(&path).unwrap();

    // Verify initial file saved
    let loaded1 = Placements::load_from(&path);
    assert_eq!(loaded1.len(), 1);
    assert!(loaded1.get(&HostId::new("host-1")).is_some());

    // Atomically replace with p2
    let mut p2 = Placements::default();
    p2.set(
        HostId::new("host-2"),
        SavedPlacement {
            output: OutputPreference::new().with_name("HDMI-1"),
            anchor: Anchor::BottomRight,
            offset: (30, 40),
        },
    );
    p2.save_to(&path).unwrap();

    // Verify replacement succeeded cleanly
    let loaded2 = Placements::load_from(&path);
    assert_eq!(loaded2.len(), 1);
    assert!(loaded2.get(&HostId::new("host-2")).is_some());
    assert!(loaded2.get(&HostId::new("host-1")).is_none());

    // Make directory read-only so that future writes/renames fail
    let mut perms = std::fs::metadata(&test_dir).unwrap().permissions();
    perms.set_mode(0o555);
    std::fs::set_permissions(&test_dir, perms).unwrap();

    let mut p3 = Placements::default();
    p3.set(
        HostId::new("host-3"),
        SavedPlacement {
            output: OutputPreference::new().with_name("DP-3"),
            anchor: Anchor::TopRight,
            offset: (50, 60),
        },
    );
    let res = p3.save_to(&path);
    assert!(res.is_err(), "saving to read-only directory must fail");

    // Restore permissions
    let mut restore_perms = std::fs::metadata(&test_dir).unwrap().permissions();
    restore_perms.set_mode(0o755);
    std::fs::set_permissions(&test_dir, restore_perms).unwrap();

    // Verify original destination file was completely untouched and preserved
    let loaded_after_error = Placements::load_from(&path);
    assert_eq!(loaded_after_error, loaded2);

    // Verify no temporary files were left behind
    let dir_entries: Vec<_> = std::fs::read_dir(&test_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(dir_entries, vec!["placements.toml"]);

    let _ = std::fs::remove_dir_all(test_dir);
}

#[test]
fn schema_dto_domain_conversion() {
    let domain_placement = SavedPlacement {
        output: OutputPreference::new()
            .with_name("HDMI-A-1")
            .with_serial("SN9876")
            .with_make("LG")
            .with_model("UltraGear")
            .with_physical_size_mm((600, 340)),
        anchor: Anchor::TopLeft,
        offset: (50, 100),
    };

    let dto = SavedPlacementDto::from(&domain_placement);
    assert_eq!(dto.anchor, AnchorDto::TopLeft);
    assert_eq!(dto.offset, (50, 100));
    assert_eq!(dto.output.name.as_deref(), Some("HDMI-A-1"));
    assert_eq!(
        dto.output
            .identity
            .as_ref()
            .and_then(|i| i.serial.as_deref()),
        Some("SN9876")
    );

    let back_to_domain = SavedPlacement::from(dto);
    assert_eq!(back_to_domain, domain_placement);
}
