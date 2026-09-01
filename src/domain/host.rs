use crate::domain::ids::{HostId, WidgetId};
use crate::domain::placement::{Anchor, PlacementDecision, SavedPlacement};

/// Generic mode defining the presentation and space reservation style of a host.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HostMode {
    /// Occupies desktop space and may reserve exclusive zones (e.g. bar).
    Cover,
    /// Floats freely over desktop content without reserving space (e.g. popups, widgets).
    Float,
    /// User-defined custom mode policy.
    Custom(String),
}

/// Generic layer assignment for a shell host surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// Exclusive zone policy for layer surface layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExclusiveZonePolicy {
    None,
    Fixed(u32),
    Auto,
}

/// Keyboard interactivity policy for surface input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardInteractivityPolicy {
    None,
    OnDemand,
    Exclusive,
}

/// Host surface sizing policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostSize {
    Exact { width: u32, height: u32 },
    OutputWidth { height: u32 },
    OutputHeight { width: u32 },
    FillOutput,
    ContentPreferred,
}

/// Declared movement policy requested by a host specification.
///
/// Distinct from runtime observed `MovementCapability`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostMovementPolicy {
    /// Host surface is intended to be movable by user interaction.
    Movable,
    /// Host surface is fixed in place and cannot be repositioned.
    Fixed,
}

/// Generic specification of a shell surface host container.
///
/// Contains spatial and layer-shell policies, but NO widget-specific fields
/// or calendar/clock variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSpec {
    pub id: HostId,
    pub namespace: String,
    pub mode: HostMode,
    pub layer: HostLayer,
    pub size: HostSize,
    pub default_anchor: Anchor,
    pub default_offset: (i32, i32),
    pub exclusive_zone: ExclusiveZonePolicy,
    pub keyboard_interactivity: KeyboardInteractivityPolicy,
    pub movement_policy: HostMovementPolicy,
    pub primary_milestone: bool,
}

impl HostSpec {
    /// Constructs a standard top bar host specification.
    pub fn top_bar(id: HostId, height: u32) -> Self {
        Self {
            id,
            namespace: "owlenshell-bar".into(),
            mode: HostMode::Cover,
            layer: HostLayer::Top,
            size: HostSize::OutputWidth { height },
            default_anchor: Anchor::TopLeft,
            default_offset: (0, 0),
            exclusive_zone: ExclusiveZonePolicy::Fixed(height),
            keyboard_interactivity: KeyboardInteractivityPolicy::None,
            movement_policy: HostMovementPolicy::Fixed,
            primary_milestone: true,
        }
    }

    /// Constructs a standard floating/movable popup host specification.
    pub fn floating(id: HostId) -> Self {
        Self {
            id,
            namespace: "owlenshell-floating".into(),
            mode: HostMode::Float,
            layer: HostLayer::Top,
            size: HostSize::ContentPreferred,
            default_anchor: Anchor::TopRight,
            default_offset: (24, 40),
            exclusive_zone: ExclusiveZonePolicy::None,
            keyboard_interactivity: KeyboardInteractivityPolicy::OnDemand,
            movement_policy: HostMovementPolicy::Movable,
            primary_milestone: false,
        }
    }
}

/// Runtime logical state of a surface host.
///
/// Acts as the single source of truth for attached widget IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostState {
    pub id: HostId,
    pub spec: Option<HostSpec>,
    pub visible: bool,
    pub attached_widgets: Vec<WidgetId>,
    pub saved_placement: Option<SavedPlacement>,
    pub current_decision: Option<PlacementDecision>,
}

impl HostState {
    pub fn new(id: HostId) -> Self {
        Self {
            id,
            spec: None,
            visible: false,
            attached_widgets: Vec::new(),
            saved_placement: None,
            current_decision: None,
        }
    }

    pub fn with_spec(spec: HostSpec) -> Self {
        Self {
            id: spec.id.clone(),
            spec: Some(spec),
            visible: false,
            attached_widgets: Vec::new(),
            saved_placement: None,
            current_decision: None,
        }
    }

    /// Attaches a widget to this host if not already attached.
    pub fn attach_widget(&mut self, widget_id: WidgetId) {
        if !self.attached_widgets.contains(&widget_id) {
            self.attached_widgets.push(widget_id);
        }
    }

    /// Detaches a widget from this host.
    pub fn detach_widget(&mut self, widget_id: &WidgetId) -> bool {
        if let Some(pos) = self.attached_widgets.iter().position(|w| w == widget_id) {
            self.attached_widgets.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns the slice of attached widget IDs.
    pub fn attached_widgets(&self) -> &[WidgetId] {
        &self.attached_widgets
    }
}
