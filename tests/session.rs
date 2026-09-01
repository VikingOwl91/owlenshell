use owlenshell::platform::session::{
    detect_session, detect_session_with, ApplicationLaunch, ChildHandle, SessionCapabilities,
    SessionDetector, SessionError, SessionIntegration, SessionManagerKind, SessionMode,
    StandaloneSession, SystemSessionDetector, UwsmSession,
};
use std::collections::HashMap;

struct MockDetector {
    result: Result<Option<SessionManagerKind>, String>,
}

impl SessionDetector for MockDetector {
    fn probe(&self) -> Result<Option<SessionManagerKind>, String> {
        self.result.clone()
    }
}

#[derive(Debug, Default)]
struct FakeManagedSession {
    ready_called: bool,
    shutdown_called: bool,
    launched: Vec<ApplicationLaunch>,
}

impl SessionIntegration for FakeManagedSession {
    fn mode(&self) -> SessionMode {
        SessionMode::Managed(SessionManagerKind::Uwsm)
    }

    fn capabilities(&self) -> SessionCapabilities {
        SessionCapabilities {
            lifecycle_notifications: true,
            managed_application_launch: true,
        }
    }

    fn notify_ready(&mut self) -> Result<(), SessionError> {
        self.ready_called = true;
        Ok(())
    }

    fn begin_shutdown(&mut self) -> Result<(), SessionError> {
        self.shutdown_called = true;
        Ok(())
    }

    fn launch(&mut self, request: ApplicationLaunch) -> Result<ChildHandle, SessionError> {
        self.launched.push(request);
        Ok(ChildHandle { id: 1337 })
    }
}

// 1. Production SystemSessionDetector safely returns Standalone without guessed env variables
#[test]
fn system_session_detector_defaults_to_standalone() {
    let detector = SystemSessionDetector;
    let probe_result = detector.probe();
    assert_eq!(probe_result, Ok(None));

    let session = detect_session();
    assert_eq!(session.mode(), SessionMode::Standalone);
    assert!(!session.capabilities().lifecycle_notifications);
    assert!(!session.capabilities().managed_application_launch);
}

// 2. Absence of managed session selects Standalone
#[test]
fn no_managed_session_detected_selects_standalone() {
    let detector = MockDetector { result: Ok(None) };
    let session = detect_session_with(&detector);
    assert_eq!(session.mode(), SessionMode::Standalone);
    assert!(!session.capabilities().lifecycle_notifications);
    assert!(!session.capabilities().managed_application_launch);
}

// 3. Detection failure/error falls back safely to Standalone
#[test]
fn detector_error_safely_falls_back_to_standalone() {
    let detector = MockDetector {
        result: Err("session manager daemon unreachable".into()),
    };
    let session = detect_session_with(&detector);
    assert_eq!(session.mode(), SessionMode::Standalone);
    assert_eq!(
        session.capabilities(),
        SessionCapabilities {
            lifecycle_notifications: false,
            managed_application_launch: false,
        }
    );
}

// 4. Injected managed UWSM detection selects Managed(Uwsm)
#[test]
fn injected_uwsm_detected_selects_managed_backend() {
    let detector = MockDetector {
        result: Ok(Some(SessionManagerKind::Uwsm)),
    };
    let session = detect_session_with(&detector);
    assert_eq!(
        session.mode(),
        SessionMode::Managed(SessionManagerKind::Uwsm)
    );
    // In Step 8, UwsmSession reports lifecycle_notifications = false until external notification is implemented
    assert!(!session.capabilities().lifecycle_notifications);
}

// 5. Standalone backend safely accepts lifecycle API as no-ops
#[test]
fn standalone_lifecycle_api_is_safe_noop() {
    let mut standalone = StandaloneSession::new();
    assert_eq!(standalone.mode(), SessionMode::Standalone);
    assert!(standalone.notify_ready().is_ok());
    assert!(standalone.begin_shutdown().is_ok());

    let launch_req = ApplicationLaunch {
        command: "true".into(),
        args: vec![],
        env: HashMap::new(),
    };
    let launch_res = standalone.launch(launch_req);
    assert!(matches!(
        launch_res,
        Err(SessionError::LaunchNotSupported(_))
    ));
}

// 6. Fake managed session receives lifecycle and launch delegation
#[test]
fn fake_managed_backend_receives_delegation() {
    let mut fake = FakeManagedSession::default();
    assert_eq!(fake.mode(), SessionMode::Managed(SessionManagerKind::Uwsm));
    assert!(fake.capabilities().lifecycle_notifications);
    assert!(fake.capabilities().managed_application_launch);

    // Lifecycle delegation
    assert!(fake.notify_ready().is_ok());
    assert!(fake.ready_called);

    // Launch delegation without spawning real processes
    let launch_req = ApplicationLaunch {
        command: "alacritty".into(),
        args: vec!["-e".into(), "htop".into()],
        env: HashMap::new(),
    };
    let handle = fake.launch(launch_req.clone()).expect("launch");
    assert_eq!(handle.id, 1337);
    assert_eq!(fake.launched, vec![launch_req]);

    assert!(fake.begin_shutdown().is_ok());
    assert!(fake.shutdown_called);
}

// 7. UwsmSession boundary implementation properties
#[test]
fn uwsm_session_boundary_contract() {
    let mut uwsm = UwsmSession::new();
    assert_eq!(uwsm.mode(), SessionMode::Managed(SessionManagerKind::Uwsm));
    assert!(!uwsm.capabilities().lifecycle_notifications);
    assert!(!uwsm.capabilities().managed_application_launch);
    assert!(uwsm.notify_ready().is_ok());
    assert!(uwsm.begin_shutdown().is_ok());

    let launch_req = ApplicationLaunch {
        command: "foot".into(),
        args: vec![],
        env: HashMap::new(),
    };
    assert!(matches!(
        uwsm.launch(launch_req),
        Err(SessionError::LaunchNotSupported(_))
    ));
}

// 8. Isolation audit: Ensure domain and runtime modules contain zero UWSM references
#[test]
fn zero_session_leakage_in_domain_and_coordinator() {
    let domain_src = std::fs::read_to_string("src/domain.rs").expect("read domain.rs");
    let placement_src = std::fs::read_to_string("src/placement.rs").expect("read placement.rs");
    let coordinator_src =
        std::fs::read_to_string("src/runtime/coordinator.rs").expect("read coordinator.rs");
    let surface_mgr_src = std::fs::read_to_string("src/platform/surface_manager.rs")
        .expect("read surface_manager.rs");

    for (name, content) in [
        ("domain.rs", domain_src),
        ("placement.rs", placement_src),
        ("coordinator.rs", coordinator_src),
        ("surface_manager.rs", surface_mgr_src),
    ] {
        assert!(
            !content.contains("uwsm")
                && !content.contains("UWSM")
                && !content.contains("SessionMode"),
            "{name} must not contain session or UWSM leakage"
        );
    }
}
