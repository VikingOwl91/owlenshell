use calloop::timer::{TimeoutAction, Timer};
use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use chrono::Datelike;
use owlenshell::compositor;
use owlenshell::instrumentation;
use owlenshell::model::{SurfaceKind, CALENDAR_KEY};
use owlenshell::persistence::Placements;
use owlenshell::runtime::{drain_and_attach, hide_kind, pump, AppShared, AppState, OwlenPlatform};
use slint::{ComponentHandle, ModelRc, VecModel};
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use smithay_client_toolkit::shm::Shm;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use wayland_client::globals::registry_queue_init;
use wayland_client::Connection;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    instrumentation::init();
    instrumentation::milestone("start");

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    instrumentation::milestone("wayland-bound");

    let shared = Rc::new(AppShared {
        conn: conn.clone(),
        qh,
        compositor,
        layer_shell,
        surfaces: RefCell::new(Vec::new()),
        new_adapters: RefCell::new(Vec::new()),
        outputs: RefCell::new(Vec::new()),
        wl_outputs: RefCell::new(Default::default()),
        store: RefCell::new(Placements::load()),
        modifiers: Cell::new(Default::default()),
        keyboard_focus: Cell::new(None),
        drag: RefCell::new(None),
        topology_dirty: Cell::new(true),
        backend: compositor::detect(),
        exit: Cell::new(false),
        first_frame: Cell::new(false),
        outputs_ready: Cell::new(false),
    });

    let mut state = AppState {
        registry: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &shared.qh),
        output_state: OutputState::new(&globals, &shared.qh),
        shm,
        pointer: None,
        keyboard: None,
        shared: shared.clone(),
    };

    slint::platform::set_platform(Box::new(OwlenPlatform {
        shared: shared.clone(),
    }))?;
    instrumentation::milestone("platform");

    let bar = Bar::new()?;
    drain_and_attach(&shared, SurfaceKind::Bar, None);
    bar.set_clock_text(clock_text());
    instrumentation::milestone("bar");

    let calendar_ui: Rc<RefCell<Option<Calendar>>> = Rc::new(RefCell::new(None));
    bar.on_open_calendar({
        let shared = shared.clone();
        let calendar_ui = calendar_ui.clone();
        move || {
            if calendar_ui.borrow().is_some() {
                hide_kind(&shared, SurfaceKind::Calendar);
                *calendar_ui.borrow_mut() = None;
                return;
            }
            let cal = Calendar::new().expect("calendar");
            let (title, weeks) = calendar_weeks();
            cal.set_month_title(title.into());
            cal.set_weeks(weeks);
            cal.on_dismiss({
                let shared = shared.clone();
                let calendar_ui = calendar_ui.clone();
                move || {
                    hide_kind(&shared, SurfaceKind::Calendar);
                    *calendar_ui.borrow_mut() = None;
                }
            });
            let _ = cal.show();
            let placement = shared.store.borrow().get(CALENDAR_KEY).cloned();
            drain_and_attach(&shared, SurfaceKind::Calendar, placement);
            *calendar_ui.borrow_mut() = Some(cal);
        }
    });
    let _ = bar.show();

    let mut event_loop: EventLoop<AppState> = EventLoop::try_new()?;
    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;
    event_loop.handle().insert_source(Timer::immediate(), {
        let bar = bar.clone_strong();
        move |_deadline, _, _state| {
            bar.set_clock_text(clock_text());
            slint::platform::update_timers_and_animations();
            TimeoutAction::ToDuration(Duration::from_secs(1))
        }
    })?;

    instrumentation::milestone("loop");
    while !state.shared.exit.get() {
        let timeout = slint::platform::duration_until_next_timer_update()
            .unwrap_or(Duration::from_secs(1))
            .min(Duration::from_secs(1));
        event_loop.dispatch(Some(timeout), &mut state)?;
        pump(&mut state)?;
        slint::platform::update_timers_and_animations();
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
