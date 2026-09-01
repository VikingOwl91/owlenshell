use calloop::timer::{TimeoutAction, Timer};
use calloop::EventLoop;
use chrono::Datelike;
use owlenshell::domain::ids::HostId;
use owlenshell::instrumentation;
use owlenshell::model::{SurfaceKind, CALENDAR_KEY};
use owlenshell::persistence::Placements;
use owlenshell::platform::{
    compositor, detect_session, OutputManager, OwlenPlatform, RenderBridge, SurfaceManager,
    WaylandPlatform,
};
use owlenshell::runtime::Coordinator;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

slint::include_modules!();

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

    let bar_id = HostId::new("bar");
    let bar = render_bridge.with_host_creation(bar_id.clone(), Bar::new)?;
    coordinator.attach_host(bar_id, SurfaceKind::Bar, None);
    bar.set_clock_text(clock_text());
    instrumentation::milestone("bar");

    let cal_id = HostId::new(CALENDAR_KEY);
    let calendar_ui: Rc<RefCell<Option<Calendar>>> = Rc::new(RefCell::new(None));
    bar.on_open_calendar({
        let coordinator = coordinator.clone();
        let calendar_ui = calendar_ui.clone();
        let render_bridge = render_bridge.clone();
        let cal_id = cal_id.clone();
        move || {
            if calendar_ui.borrow().is_some() {
                coordinator.hide_host(&cal_id);
                *calendar_ui.borrow_mut() = None;
                render_bridge.remove_adapter(&cal_id);
                return;
            }
            let cal = render_bridge
                .with_host_creation(cal_id.clone(), Calendar::new)
                .expect("calendar");
            let (title, weeks) = calendar_weeks();
            cal.set_month_title(title.into());
            cal.set_weeks(weeks);
            cal.on_dismiss({
                let coordinator = coordinator.clone();
                let calendar_ui = calendar_ui.clone();
                let render_bridge = render_bridge.clone();
                let cal_id = cal_id.clone();
                move || {
                    coordinator.hide_host(&cal_id);
                    *calendar_ui.borrow_mut() = None;
                    render_bridge.remove_adapter(&cal_id);
                }
            });
            let _ = cal.show();
            let placement = coordinator.placements_store().get(&cal_id).cloned();
            coordinator.attach_host(cal_id.clone(), SurfaceKind::Calendar, placement);
            *calendar_ui.borrow_mut() = Some(cal);
        }
    });
    let _ = bar.show();

    let mut event_loop: EventLoop<WaylandPlatform> = EventLoop::try_new()?;
    let _ = platform.insert_into_event_loop(&event_loop)?;
    event_loop.handle().insert_source(Timer::immediate(), {
        let bar = bar.clone_strong();
        move |_deadline, _, _platform| {
            bar.set_clock_text(clock_text());
            slint::platform::update_timers_and_animations();
            TimeoutAction::ToDuration(Duration::from_secs(1))
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

fn clock_text() -> slint::SharedString {
    chrono::Local::now().format("%H:%M:%S").to_string().into()
}

fn calendar_weeks() -> (String, ModelRc<DayRow>) {
    let now = chrono::Local::now().date_naive();
    let title = now.format("%B %Y").to_string();
    let first = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
    let start = first.weekday().num_days_from_monday() as usize;
    let dim = days_in_month(now.year(), now.month());
    let mut cells = [0i32; 42];
    for d in 1..=dim {
        cells[start + d as usize - 1] = d as i32;
    }
    let rows: Vec<DayRow> = cells
        .chunks(7)
        .map(|c| DayRow {
            d0: c[0],
            d1: c[1],
            d2: c[2],
            d3: c[3],
            d4: c[4],
            d5: c[5],
            d6: c[6],
        })
        .collect();
    (title, Rc::new(VecModel::from(rows)).into())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
