use crate::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use crate::domain::ids::OutputId;
use crate::domain::placement::{Anchor, SavedPlacement};
use crate::model::{SurfaceKind, BAR_HEIGHT};
use crate::placement::{clamp_offset, margins_from_offset, MIN_VISIBLE};
use crate::platform::SurfaceSpec;
use slint::LogicalSize;

pub use crate::model::HANDLE_HEIGHT;

/// Temporary compatibility check for Bar host kind.
pub fn is_bar_host(kind: SurfaceKind) -> bool {
    kind == SurfaceKind::Bar
}

/// Temporary compatibility check for Calendar host kind.
pub fn is_calendar_host(kind: SurfaceKind) -> bool {
    kind == SurfaceKind::Calendar
}

/// Helper to find bar output ID from surface manager.
pub fn bar_output_id(
    surface_manager: &crate::platform::SurfaceManager,
    bar_host_id: &crate::domain::ids::HostId,
) -> Option<OutputId> {
    surface_manager.output_id_for_host(bar_host_id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachGeometry {
    pub anchor: Anchor,
    pub margins: (i32, i32, i32, i32),
    pub requested_size: (u32, u32),
    pub effective_widget_size: (i32, i32),
    pub offset: (i32, i32),
}

pub fn compute_attach_geometry(
    kind: SurfaceKind,
    placement: Option<&SavedPlacement>,
    preferred: LogicalSize,
    output_logical_size: Option<(i32, i32)>,
) -> AttachGeometry {
    match kind {
        SurfaceKind::Bar => AttachGeometry {
            anchor: Anchor::TopLeft,
            margins: (0, 0, 0, 0),
            requested_size: (0, BAR_HEIGHT),
            effective_widget_size: (0, BAR_HEIGHT as i32),
            offset: (0, 0),
        },
        SurfaceKind::Calendar => {
            let anchor = placement.map(|p| p.anchor).unwrap_or(Anchor::TopRight);
            let raw_offset = placement.map(|p| p.offset).unwrap_or((24, 40));
            let widget_w = preferred.width.max(1.0) as i32;
            let widget_h = preferred.height.max(1.0) as i32;
            let offset = clamp_offset(
                anchor,
                raw_offset,
                (widget_w, widget_h),
                output_logical_size,
                None,
                MIN_VISIBLE,
            );
            let margins = margins_from_offset(anchor, offset);
            let w = widget_w as u32;
            let h = widget_h as u32;
            AttachGeometry {
                anchor,
                margins,
                requested_size: (w, h),
                effective_widget_size: (widget_w, widget_h),
                offset,
            }
        }
    }
}

pub fn surface_spec_for(kind: SurfaceKind) -> SurfaceSpec {
    match kind {
        SurfaceKind::Bar => SurfaceSpec {
            namespace: "owlenshell-bar".into(),
            layer: HostLayer::Top,
            mode: HostMode::Cover,
            size: HostSize::OutputWidth { height: BAR_HEIGHT },
            exclusive_zone: ExclusiveZonePolicy::Fixed(BAR_HEIGHT),
            keyboard_interactivity: KeyboardInteractivityPolicy::None,
        },
        SurfaceKind::Calendar => SurfaceSpec {
            namespace: "owlenshell-calendar".into(),
            layer: HostLayer::Top,
            mode: HostMode::Float,
            size: HostSize::ContentPreferred,
            exclusive_zone: ExclusiveZonePolicy::None,
            keyboard_interactivity: KeyboardInteractivityPolicy::OnDemand,
        },
    }
}
