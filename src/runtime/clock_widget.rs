use crate::domain::ids::{HostId, WidgetId};
use crate::domain::widget::{WidgetAction, WidgetSpec};
use std::time::Duration;

/// Formats the current local time for standard Clock widget display (e.g. "15:04:05").
pub fn clock_text() -> slint::SharedString {
    chrono::Local::now().format("%H:%M:%S").to_string().into()
}

/// Controller for the Clock widget.
///
/// Encapsulates clock display state, time formatting, action target, and widget identity.
/// Contains zero Wayland, layer-shell, or native surface dependencies.
#[derive(Debug, Clone)]
pub struct ClockWidgetController {
    spec: WidgetSpec,
    toggle_target: Option<HostId>,
}

impl ClockWidgetController {
    /// Creates a new `ClockWidgetController` with the given widget ID and optional action target.
    pub fn new(id: WidgetId, toggle_target: Option<HostId>) -> Self {
        Self {
            spec: WidgetSpec::clock(id),
            toggle_target,
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

    /// Generates the current formatted clock text.
    pub fn current_text(&self) -> slint::SharedString {
        clock_text()
    }

    /// Standard clock update interval (1 second).
    pub fn update_interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// Emits a generic `WidgetAction` when the clock is clicked / activated.
    pub fn handle_click(&self) -> Option<WidgetAction> {
        self.toggle_target
            .as_ref()
            .cloned()
            .map(WidgetAction::ToggleHost)
    }
}
