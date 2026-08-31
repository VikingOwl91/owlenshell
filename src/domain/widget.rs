use crate::domain::ids::WidgetId;

/// Generic specification of a logical widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetSpec {
    pub id: WidgetId,
    pub title: String,
    pub enabled_by_default: bool,
}

/// Runtime state of a logical widget.
///
/// Host attachment is canonically tracked by `HostState.attached_widgets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetState {
    pub id: WidgetId,
    pub enabled: bool,
}

impl WidgetState {
    pub fn new(id: WidgetId) -> Self {
        Self { id, enabled: true }
    }
}
