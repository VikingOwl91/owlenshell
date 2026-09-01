use owlenshell::domain::ids::HostId;
use owlenshell::platform::render_bridge::{RenderBridge, RenderBridgeError};
use owlenshell::ui::{Bar, Calendar};
use slint::platform::{Platform, WindowAdapter};
use slint::PlatformError;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

thread_local! {
    static CURRENT_BRIDGE: RefCell<Option<Rc<RenderBridge>>> = const { RefCell::new(None) };
    static THREAD_INIT: Once = const { Once::new() };
}

struct GlobalTestPlatform;

impl Platform for GlobalTestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        CURRENT_BRIDGE.with(|bridge| {
            let bridge = bridge.borrow();
            let bridge = bridge.as_ref().expect(
                "CURRENT_BRIDGE must be set on this test thread before creating Slint components",
            );
            let adapter = bridge.create_window_adapter()?;
            Ok(adapter as Rc<dyn WindowAdapter>)
        })
    }
}

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn setup_test_bridge() -> (std::sync::MutexGuard<'static, ()>, Rc<RenderBridge>) {
    let guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bridge = RenderBridge::new();
    CURRENT_BRIDGE.with(|b| {
        *b.borrow_mut() = Some(bridge.clone());
    });
    THREAD_INIT.with(|init| {
        init.call_once(|| {
            let _ = slint::platform::set_platform(Box::new(GlobalTestPlatform));
        });
    });
    (guard, bridge)
}

#[test]
fn pinned_slint_bar_constructor_creates_adapter_synchronously() {
    let (_lock, bridge) = setup_test_bridge();
    let host_id = HostId::new("bar");

    assert_eq!(bridge.adapter_count(), 0);
    assert!(bridge.pending_creation().is_none());

    let bar = bridge
        .with_host_creation(host_id.clone(), Bar::new)
        .expect("Bar constructor must succeed within with_host_creation");

    // 1. Adapter registered for host_id
    assert_eq!(bridge.adapter_count(), 1);
    assert!(bridge.has_adapter(&host_id));
    assert!(bridge.pending_creation().is_none());

    let adapter = bridge
        .get_adapter(&host_id)
        .expect("adapter must exist for host_id");
    assert_eq!(adapter.host_id(), &host_id);
    assert_eq!(&adapter.host_id, &host_id);

    // Component is usable
    bar.set_clock_text("12:34:56".into());
}

#[test]
fn pinned_slint_calendar_constructor_creates_adapter_synchronously() {
    let (_lock, bridge) = setup_test_bridge();
    let host_id = HostId::new("calendar");

    assert_eq!(bridge.adapter_count(), 0);
    assert!(bridge.pending_creation().is_none());

    let calendar = bridge
        .with_host_creation(host_id.clone(), Calendar::new)
        .expect("Calendar constructor must succeed within with_host_creation");

    // 1. Adapter registered for host_id
    assert_eq!(bridge.adapter_count(), 1);
    assert!(bridge.has_adapter(&host_id));
    assert!(bridge.pending_creation().is_none());

    let adapter = bridge
        .get_adapter(&host_id)
        .expect("adapter must exist for host_id");
    assert_eq!(adapter.host_id(), &host_id);

    calendar.set_month_title("September 2026".into());
}

#[test]
fn host_id_binding_isolation_and_lookup() {
    let (_lock, bridge) = setup_test_bridge();
    let host_a = HostId::new("host-a");
    let host_b = HostId::new("host-b");

    let _bar = bridge
        .with_host_creation(host_a.clone(), Bar::new)
        .expect("host_a creation");
    let _cal = bridge
        .with_host_creation(host_b.clone(), Calendar::new)
        .expect("host_b creation");

    assert_eq!(bridge.adapter_count(), 2);
    assert!(bridge.has_adapter(&host_a));
    assert!(bridge.has_adapter(&host_b));

    let adapter_a = bridge.get_adapter(&host_a).expect("adapter_a");
    let adapter_b = bridge.get_adapter(&host_b).expect("adapter_b");

    assert_eq!(adapter_a.host_id(), &host_a);
    assert_eq!(adapter_b.host_id(), &host_b);
    assert!(!Rc::ptr_eq(&adapter_a, &adapter_b));
}

#[test]
fn create_window_adapter_without_pending_host_id_fails() {
    let bridge = RenderBridge::new();
    let result = bridge.create_window_adapter();
    assert!(result.is_err());
}

#[test]
fn nested_creation_scope_is_rejected() {
    let (_lock, bridge) = setup_test_bridge();
    let host_a = HostId::new("host-a");
    let host_b = HostId::new("host-b");

    let nested_result = bridge.with_host_creation(host_a.clone(), || {
        // Attempting to nest with_host_creation inside constructor
        let inner = bridge.with_host_creation(host_b.clone(), Bar::new);
        assert_eq!(inner.err(), Some(RenderBridgeError::NestedCreation));
        Bar::new()
    });

    assert!(nested_result.is_ok());
    assert!(bridge.has_adapter(&host_a));
    assert!(!bridge.has_adapter(&host_b));
}

#[test]
fn duplicate_host_id_registration_is_rejected() {
    let (_lock, bridge) = setup_test_bridge();
    let host = HostId::new("unique-host");

    let first = bridge.with_host_creation(host.clone(), Bar::new);
    assert!(first.is_ok());

    let second = bridge.with_host_creation(host.clone(), Bar::new);
    assert_eq!(second.err(), Some(RenderBridgeError::DuplicateHost(host)));
}

#[test]
fn second_adapter_callback_for_one_token_is_rejected() {
    let (_lock, bridge) = setup_test_bridge();
    let host = HostId::new("single-token-host");

    let res = bridge.with_host_creation(host.clone(), || {
        // First callback consumes the token
        let adapter1 = bridge.create_window_adapter().map_err(|e| e.to_string())?;
        assert_eq!(adapter1.host_id(), &host);

        // Second callback without another token must fail
        let adapter2 = bridge.create_window_adapter();
        assert!(
            adapter2.is_err(),
            "second callback for same token must fail"
        );

        Ok::<(), String>(())
    });

    assert!(res.is_ok());
    assert_eq!(bridge.adapter_count(), 1);
}

#[test]
fn adapter_rolled_back_if_constructor_errors_after_registration() {
    let (_lock, bridge) = setup_test_bridge();
    let host = HostId::new("error-after-reg-host");

    let result = bridge.with_host_creation(host.clone(), || {
        // 1. Consume token and register adapter successfully
        let adapter = bridge.create_window_adapter().map_err(|e| e.to_string())?;
        assert_eq!(adapter.host_id(), &host);

        // 2. But constructor then returns an error
        Err::<Bar, String>("constructor failed after registration".to_string())
    });

    // 3. Creation scope returned error
    assert!(result.is_err());
    // 4. Registry contains no ghost adapter for host
    assert!(!bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 0);
    // 5. Pending state is clear
    assert!(bridge.pending_creation().is_none());

    // 6. Host can be retried and real component construction succeeds
    let retry = bridge.with_host_creation(host.clone(), Bar::new);
    assert!(retry.is_ok());
    assert!(bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 1);
}

#[test]
fn adapter_rolled_back_if_constructor_panics_after_registration() {
    let (_lock, bridge) = setup_test_bridge();
    let host = HostId::new("panic-after-reg-host");

    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = bridge.with_host_creation(host.clone(), || -> Result<Bar, String> {
            // 1. Consume token and register adapter successfully
            let adapter = bridge.create_window_adapter().map_err(|e| e.to_string())?;
            assert_eq!(adapter.host_id(), &host);

            // 2. Constructor panics after adapter is registered
            panic!("constructor panicked after adapter registration");
        });
    }));

    // 3. Panic caught
    assert!(panic_result.is_err());
    // 4. Ghost adapter rolled back
    assert!(!bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 0);
    // 5. Pending state is clear
    assert!(bridge.pending_creation().is_none());

    // 6. Subsequent creation for the same HostId succeeds cleanly
    let retry = bridge.with_host_creation(host.clone(), Bar::new);
    assert!(retry.is_ok());
    assert!(bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 1);
}

#[test]
fn constructor_success_without_token_consumption_is_rejected() {
    let (_lock, bridge) = setup_test_bridge();
    let host = HostId::new("no-token-consumed-host");

    // Constructor returns Ok(()) without creating/consuming adapter
    let result = bridge.with_host_creation(host.clone(), || Ok::<(), String>(()));

    assert_eq!(
        result.err(),
        Some(RenderBridgeError::TokenNotConsumed(host.clone()))
    );
    assert!(!bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 0);
    assert!(bridge.pending_creation().is_none());

    // HostId remains reusable for real component creation
    let retry = bridge.with_host_creation(host.clone(), Bar::new);
    assert!(retry.is_ok());
    assert!(bridge.has_adapter(&host));
    assert_eq!(bridge.adapter_count(), 1);
}

#[test]
fn native_recreation_transition_preserves_adapter_and_triggers_no_new_registration() {
    let (_lock, bridge) = setup_test_bridge();
    let host_id = HostId::new("calendar");

    // 1. Initial creation through RenderBridge
    let cal = bridge
        .with_host_creation(host_id.clone(), Calendar::new)
        .expect("initial component creation");
    let initial_adapter = bridge
        .get_adapter(&host_id)
        .expect("adapter must be registered in bridge");
    assert_eq!(bridge.adapter_count(), 1);

    // 2. Initialize runtime surface state with active native handles in SurfaceManager
    let mut surface = owlenshell::runtime::SurfaceState {
        host_id: host_id.clone(),
        kind: owlenshell::model::SurfaceKind::Calendar,
        adapter: initial_adapter.clone(),
        placement: None,
        visible: true,
    };

    let mut surface_manager = owlenshell::platform::SurfaceManager::new();
    let token1 = surface_manager
        .generation_registry_mut_for_test()
        .allocate(host_id.clone());
    assert_eq!(token1.generation.raw(), 1);

    // 3. Execute actual runtime retirement transition (used on closed/disconnect/target-change)
    let retired_token = surface_manager.retire(&host_id);
    assert_eq!(retired_token, Some(token1));

    // Invariant: Native token retired, logical visibility preserved
    assert!(surface.visible);
    assert!(!surface_manager.has_native_instance(&host_id));
    assert_eq!(surface_manager.current_token(&host_id), None);

    // 4. Execute actual runtime recreation preparation transition (used by attach_host on reattach)
    let reattached_adapter = bridge
        .get_adapter(&surface.host_id)
        .expect("host adapter must survive native retirement");
    surface.prepare_recreation(reattached_adapter, None);

    // 5. Invariant assertions:
    // - The exact same adapter Rc survives
    assert!(Rc::ptr_eq(&surface.adapter, &initial_adapter));
    // - Logical visibility remains true
    assert!(surface.visible);
    // - Exactly 1 adapter remains in RenderBridge registry
    assert_eq!(bridge.adapter_count(), 1);
    // - No pending creation left open
    assert!(bridge.pending_creation().is_none());
    // - No unexpected create_window_adapter callback can succeed without a token
    assert!(bridge.create_window_adapter().is_err());
    // - Component remains live and usable
    cal.set_month_title("November 2026".into());
}
