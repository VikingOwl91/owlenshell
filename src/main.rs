use calloop::timer::{TimeoutAction, Timer};
use calloop::EventLoop;
use owlenshell::domain::default_shell::{
    ShellSpec, DEFAULT_BAR_HOST_ID, DEFAULT_CALENDAR_HOST_ID, DEFAULT_CALENDAR_WIDGET_ID,
    DEFAULT_CLOCK_WIDGET_ID,
};
use owlenshell::domain::ids::{HostId, WidgetId};
use owlenshell::instrumentation;
use owlenshell::persistence::Placements;
use owlenshell::platform::{
    compositor, detect_session, OutputManager, OwlenPlatform, RenderBridge, SurfaceManager,
    WaylandPlatform,
};
use owlenshell::runtime::{
    CalendarDayRow, CalendarWidgetController, ClockWidgetController, Coordinator,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

slint::include_modules!();

/// Converts domain `CalendarDayRow` slices into the Slint `ModelRc<DayRow>` presentation model.
fn bind_calendar_weeks(rows: &[CalendarDayRow]) -> ModelRc<DayRow> {
    let slint_rows: Vec<DayRow> = rows
        .iter()
        .map(|c| DayRow {
            d0: c.d0,
            d1: c.d1,
            d2: c.d2,
            d3: c.d3,
            d4: c.d4,
            d5: c.d5,
            d6: c.d6,
        })
        .collect();
    Rc::new(VecModel::from(slint_rows)).into()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    instrumentation::init();
    instrumentation::milestone("start");

    let mut session = detect_session();
    tracing::info!(mode = ?session.mode(), "session backend selected");

    let output_manager = Rc::new(RefCell::new(OutputManager::new(compositor::detect())));
    let surface_manager = Rc::new(RefCell::new(SurfaceManager::new()));
    let render_bridge = RenderBridge::new();

    let mut platform = WaylandPlatform::connect(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
    )?;

    let coordinator = Rc::new(Coordinator::new(
        output_manager.clone(),
        surface_manager.clone(),
        render_bridge.clone(),
        Placements::load(),
    ));

    slint::platform::set_platform(Box::new(OwlenPlatform::new(render_bridge.clone())))?;
    instrumentation::milestone("platform");

    // 1. Establish the declarative default shell composition and register with Coordinator
    let shell_spec = ShellSpec::default_shell();
    coordinator.apply_shell_spec(&shell_spec);

    let bar_id = HostId::new(DEFAULT_BAR_HOST_ID);
    let cal_id = HostId::new(DEFAULT_CALENDAR_HOST_ID);
    let clock_id = WidgetId::new(DEFAULT_CLOCK_WIDGET_ID);
    let cal_widget_id = WidgetId::new(DEFAULT_CALENDAR_WIDGET_ID);

    // 2. Initialize ClockWidget controller sourced from composition
    let clock_controller = Rc::new(ClockWidgetController::new(clock_id, Some(cal_id.clone())));

    // 3. Initialize CalendarWidget controller sourced from composition
    let calendar_controller = Rc::new(RefCell::new(CalendarWidgetController::new(
        cal_widget_id,
        cal_id.clone(),
    )));

    // 4. Materialize the initial BarHost UI component
    let bar = render_bridge.with_host_creation(bar_id.clone(), Bar::new)?;
    bar.set_clock_text(clock_controller.current_text());
    instrumentation::milestone("bar");

    // 5. Container for temporary Slint Calendar UI handle
    let calendar_ui: Rc<RefCell<Option<Calendar>>> = Rc::new(RefCell::new(None));

    // Connect clock interaction to generic HostCommand dispatch
    bar.on_open_calendar({
        let coordinator = coordinator.clone();
        let clock_controller = clock_controller.clone();
        let calendar_controller = calendar_controller.clone();
        let calendar_ui = calendar_ui.clone();
        let render_bridge = render_bridge.clone();
        let cal_id = cal_id.clone();
        move || {
            if let Some(action) = clock_controller.handle_click() {
                let was_visible = coordinator
                    .get_host(&cal_id)
                    .map(|h| h.visible)
                    .unwrap_or(false);

                // Generic logical state transition executed via Coordinator
                coordinator.dispatch_action(action);

                let is_visible = coordinator
                    .get_host(&cal_id)
                    .map(|h| h.visible)
                    .unwrap_or(false);

                // Materialize / tear down Calendar UI bound to controller state
                match (was_visible, is_visible) {
                    (true, false) => {
                        *calendar_ui.borrow_mut() = None;
                        render_bridge.remove_adapter(&cal_id);
                    }
                    (false, true) => {
                        let cal = render_bridge
                            .with_host_creation(cal_id.clone(), Calendar::new)
                            .expect("calendar");
                        let controller = calendar_controller.borrow();
                        cal.set_month_title(controller.month_title().into());
                        cal.set_weeks(bind_calendar_weeks(&controller.week_rows()));
                        cal.on_dismiss({
                            let coordinator = coordinator.clone();
                            let calendar_controller = calendar_controller.clone();
                            let calendar_ui = calendar_ui.clone();
                            let render_bridge = render_bridge.clone();
                            let cal_id = cal_id.clone();
                            move || {
                                let dismiss_action = calendar_controller.borrow().handle_dismiss();
                                coordinator.dispatch_action(dismiss_action);
                                *calendar_ui.borrow_mut() = None;
                                render_bridge.remove_adapter(&cal_id);
                            }
                        });
                        let _ = cal.show();
                        *calendar_ui.borrow_mut() = Some(cal);
                    }
                    _ => {}
                }
            }
        }
    });
    let _ = bar.show();

    let mut event_loop: EventLoop<WaylandPlatform> = EventLoop::try_new()?;
    let _ = platform.insert_into_event_loop(&event_loop)?;
    event_loop.handle().insert_source(Timer::immediate(), {
        let bar = bar.clone_strong();
        let clock_controller = clock_controller.clone();
        move |_deadline, _, _platform| {
            bar.set_clock_text(clock_controller.current_text());
            slint::platform::update_timers_and_animations();
            TimeoutAction::ToDuration(clock_controller.update_interval())
        }
    })?;

    let mut notified_ready = false;
    instrumentation::milestone("loop");
    while !coordinator.should_exit() {
        let timeout = slint::platform::duration_until_next_timer_update()
            .unwrap_or(Duration::from_secs(1))
            .min(Duration::from_secs(1));
        event_loop.dispatch(Some(timeout), &mut platform)?;
        coordinator.pump(&mut platform)?;

        if coordinator.first_frame_fired() && !notified_ready {
            if let Err(err) = session.notify_ready() {
                tracing::warn!(%err, "failed to notify session readiness");
            }
            notified_ready = true;
        }

        slint::platform::update_timers_and_animations();
    }

    if let Err(err) = session.begin_shutdown() {
        tracing::warn!(%err, "failed to notify session shutdown");
    }

    Ok(())
}
