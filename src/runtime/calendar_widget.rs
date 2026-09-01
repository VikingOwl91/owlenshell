use crate::domain::ids::{HostId, WidgetId};
use crate::domain::widget::{HostCommand, WidgetAction, WidgetSpec};
use chrono::{Datelike, NaiveDate};

/// Presentation data for a single day row in the Slint calendar view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDayRow {
    pub d0: i32,
    pub d1: i32,
    pub d2: i32,
    pub d3: i32,
    pub d4: i32,
    pub d5: i32,
    pub d6: i32,
}

/// Computes the number of days in a given month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
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

/// Controller for the Calendar widget.
///
/// Encapsulates calendar content generation, selected date/month, action target, and widget identity.
/// Contains zero Wayland, layer-shell, or native surface dependencies.
#[derive(Debug, Clone)]
pub struct CalendarWidgetController {
    spec: WidgetSpec,
    selected_year: i32,
    selected_month: u32,
    host_id: HostId,
}

impl CalendarWidgetController {
    /// Creates a new `CalendarWidgetController` for the current system date and target host.
    pub fn new(id: WidgetId, host_id: HostId) -> Self {
        let now = chrono::Local::now().date_naive();
        Self {
            spec: WidgetSpec::calendar(id),
            selected_year: now.year(),
            selected_month: now.month(),
            host_id,
        }
    }

    /// Creates a new `CalendarWidgetController` with explicit year and month.
    pub fn with_date(id: WidgetId, host_id: HostId, year: i32, month: u32) -> Self {
        Self {
            spec: WidgetSpec::calendar(id),
            selected_year: year,
            selected_month: month,
            host_id,
        }
    }

    /// Returns a reference to the underlying `WidgetSpec`.
    pub fn spec(&self) -> &WidgetSpec {
        &self.spec
    }

    /// Returns the widget's unique identifier.
    pub fn id(&self) -> &WidgetId {
        &self.spec.id
    }

    /// Returns the host identifier associated with this calendar popup.
    pub fn host_id(&self) -> &HostId {
        &self.host_id
    }

    /// Returns the currently displayed year.
    pub fn selected_year(&self) -> i32 {
        self.selected_year
    }

    /// Returns the currently displayed month (1..=12).
    pub fn selected_month(&self) -> u32 {
        self.selected_month
    }

    /// Sets the displayed year and month.
    pub fn set_date(&mut self, year: i32, month: u32) {
        self.selected_year = year;
        self.selected_month = month;
    }

    /// Formats the current month title (e.g. "September 2026").
    pub fn month_title(&self) -> String {
        let date = NaiveDate::from_ymd_opt(self.selected_year, self.selected_month, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        date.format("%B %Y").to_string()
    }

    /// Generates the standard 6-week (42 cell) day grid representation.
    pub fn week_rows(&self) -> Vec<CalendarDayRow> {
        let first = NaiveDate::from_ymd_opt(self.selected_year, self.selected_month, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let start = first.weekday().num_days_from_monday() as usize;
        let dim = days_in_month(self.selected_year, self.selected_month);
        let mut cells = [0i32; 42];
        for d in 1..=dim {
            cells[start + d as usize - 1] = d as i32;
        }
        cells
            .chunks(7)
            .map(|c| CalendarDayRow {
                d0: c[0],
                d1: c[1],
                d2: c[2],
                d3: c[3],
                d4: c[4],
                d5: c[5],
                d6: c[6],
            })
            .collect()
    }

    /// Emits a generic `WidgetAction` to dismiss / hide the calendar host.
    pub fn handle_dismiss(&self) -> WidgetAction {
        WidgetAction::Command(HostCommand::Hide(self.host_id.clone()))
    }
}
