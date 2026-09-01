use crate::domain::ids::{HostId, WidgetId};

/// Generic logical action emitted by an interactive widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetAction {
    /// Toggles the logical visibility of a target host.
    ToggleHost(HostId),
}

/// Classification of first-party widget content kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Clock,
    Calendar,
    Custom,
}

/// Generic specification of a logical widget.
///
/// Contains widget metadata and identity, but NO host layout,
/// layer-shell anchor, or Wayland surface policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetSpec {
    pub id: WidgetId,
    pub kind: WidgetKind,
    pub title: String,
    pub enabled_by_default: bool,
}

impl WidgetSpec {
    pub fn new(id: WidgetId, kind: WidgetKind, title: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            title: title.into(),
            enabled_by_default: true,
        }
    }

    /// Constructs a standard clock widget specification.
    pub fn clock(id: WidgetId) -> Self {
        Self {
            id,
            kind: WidgetKind::Clock,
            title: "Clock".into(),
            enabled_by_default: true,
        }
    }

    /// Constructs a standard calendar widget specification.
    pub fn calendar(id: WidgetId) -> Self {
        Self {
            id,
            kind: WidgetKind::Calendar,
            title: "Calendar".into(),
            enabled_by_default: true,
        }
    }
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
