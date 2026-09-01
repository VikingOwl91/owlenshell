use crate::domain::ids::OutputId;
use crate::domain::output::{OutputIdentity, OutputSnapshot};
use crate::domain::placement::{
    Anchor, DeferredSurface, PlacedSurface, PlacementDecision, ResolveReason, SavedPlacement,
};

pub const MIN_VISIBLE: i32 = 32;

/// Context supplied to placement resolution from the runtime environment.
#[derive(Debug, Clone, Default)]
pub struct PlacementContext {
    pub spawn_output_id: Option<OutputId>,
    pub preferred_name: Option<String>,
    pub focused_output_id: Option<OutputId>,
}

/// Generic host constraints for sizing, work area, and reachability.
#[derive(Debug, Clone)]
pub struct HostPlacementConstraints {
    pub min_visible: i32,
    pub widget_size: (i32, i32),
    pub work_area: Option<((i32, i32), (i32, i32))>,
}

impl Default for HostPlacementConstraints {
    fn default() -> Self {
        Self {
            min_visible: MIN_VISIBLE,
            widget_size: (MIN_VISIBLE, MIN_VISIBLE),
            work_area: None,
        }
    }
}

/// Result of committing a user drag move interaction.
///
/// Contains both the ephemeral runtime target OutputId and the persistable SavedPlacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveCommitResult {
    pub target_output_id: OutputId,
    pub saved_placement: SavedPlacement,
    pub anchor: Anchor,
    pub offset: (i32, i32),
}

/// Pure decision engine for placement resolution and semantic move commits.
pub struct PlacementEngine;

impl PlacementEngine {
    /// Resolves runtime placement from saved intent, active outputs, context, and host constraints.
    ///
    /// This function is entirely pure and has no side effects:
    /// - It does not create Wayland surfaces.
    /// - It does not mutate persistence.
    /// - It does not interact with Slint.
    /// - It contains no widget-kind branching.
    pub fn resolve(
        saved: Option<&SavedPlacement>,
        outputs: &[OutputSnapshot],
        ctx: &PlacementContext,
        constraints: &HostPlacementConstraints,
    ) -> PlacementDecision {
        if outputs.is_empty() {
            let anchor = saved.map(|s| s.anchor).unwrap_or(Anchor::TopLeft);
            let offset = saved.map(|s| s.offset).unwrap_or((0, 0));
            return PlacementDecision::Deferred(DeferredSurface {
                anchor,
                offset,
                margins: margins_from_offset(anchor, offset),
                reason: ResolveReason::Deferred,
            });
        }

        let mut matched_target: Option<OutputId> = None;
        let mut match_reason = ResolveReason::Deferred;
        let mut is_fallback = false;

        // 1. Exact connector / name match from saved preference
        if let Some(pref) = saved.map(|s| &s.output) {
            if let Some(name) = pref.name.as_deref() {
                if let Some(o) = outputs.iter().find(|o| o.name.as_deref() == Some(name)) {
                    matched_target = Some(o.id);
                    match_reason = ResolveReason::ExactName;
                    is_fallback = false;
                }
            }

            // 2. Strong serial match (unique match required)
            if matched_target.is_none() {
                if let Some(serial) = pref.serial() {
                    if !serial.is_empty() {
                        let matches: Vec<&OutputSnapshot> = outputs
                            .iter()
                            .filter(|o| o.serial() == Some(serial))
                            .collect();
                        if matches.len() == 1 {
                            matched_target = Some(matches[0].id);
                            match_reason = ResolveReason::Serial;
                            is_fallback = false;
                        }
                    }
                }
            }

            // 3. Hardware fingerprint match (make/model/physical size, unique match required)
            if matched_target.is_none() {
                if let Some(ident) = &pref.identity {
                    if ident.make.is_some()
                        || ident.model.is_some()
                        || ident.physical_size_mm.is_some()
                    {
                        let matches: Vec<&OutputSnapshot> = outputs
                            .iter()
                            .filter(|o| fingerprint_matches(ident, o))
                            .collect();
                        if matches.len() == 1 {
                            matched_target = Some(matches[0].id);
                            match_reason = ResolveReason::Fingerprint;
                            is_fallback = false;
                        }
                    }
                }
            }

            // 4. Configured logical role match (matching against actual logical_role, not connector name)
            if matched_target.is_none() {
                if let Some(role) = pref.logical_role.as_deref() {
                    let matches: Vec<&OutputSnapshot> = outputs
                        .iter()
                        .filter(|o| o.logical_role.as_deref() == Some(role))
                        .collect();
                    if matches.len() == 1 {
                        matched_target = Some(matches[0].id);
                        match_reason = ResolveReason::LogicalRole;
                        is_fallback = false;
                    }
                }
            }
        }

        // If saved preference was not matched, any subsequent resolution is a fallback
        if matched_target.is_none() {
            let has_saved = saved.is_some();

            // 5. Spawn output
            if let Some(id) = ctx.spawn_output_id {
                if outputs.iter().any(|o| o.id == id) {
                    matched_target = Some(id);
                    match_reason = ResolveReason::SpawnOutput;
                    is_fallback = has_saved;
                }
            }

            // 6. Adapter-reported preferred/focused output by name
            if matched_target.is_none() {
                if let Some(name) = ctx.preferred_name.as_deref() {
                    if let Some(o) = outputs.iter().find(|o| o.name.as_deref() == Some(name)) {
                        matched_target = Some(o.id);
                        match_reason = ResolveReason::Preferred;
                        is_fallback = has_saved;
                    }
                }
            }

            // 6b. Focused output id
            if matched_target.is_none() {
                if let Some(id) = ctx.focused_output_id {
                    if outputs.iter().any(|o| o.id == id) {
                        matched_target = Some(id);
                        match_reason = ResolveReason::Preferred;
                        is_fallback = has_saved;
                    }
                }
            }

            // 6c. Output with preferred flag
            if matched_target.is_none() {
                if let Some(o) = outputs.iter().find(|o| o.preferred) {
                    matched_target = Some(o.id);
                    match_reason = ResolveReason::Preferred;
                    is_fallback = has_saved;
                }
            }

            // 7. Deterministic active output fallback (ordered by stable properties, runtime OutputId only as tie-breaker)
            if matched_target.is_none() {
                let chosen = outputs
                    .iter()
                    .min_by_key(|o| output_fallback_key(o))
                    .unwrap();
                matched_target = Some(chosen.id);
                match_reason = ResolveReason::Any;
                is_fallback = has_saved;
            }
        }

        let target_output_id = match matched_target {
            Some(id) => id,
            None => {
                let anchor = saved.map(|s| s.anchor).unwrap_or(Anchor::TopLeft);
                let offset = saved.map(|s| s.offset).unwrap_or((0, 0));
                return PlacementDecision::Deferred(DeferredSurface {
                    anchor,
                    offset,
                    margins: margins_from_offset(anchor, offset),
                    reason: match_reason,
                });
            }
        };

        let target_output = outputs.iter().find(|o| o.id == target_output_id).unwrap();
        let anchor = saved.map(|s| s.anchor).unwrap_or(Anchor::TopRight);
        let raw_offset = saved.map(|s| s.offset).unwrap_or((0, 0));

        let out_size = target_output.logical_size;
        let clamped = clamp_offset(
            anchor,
            raw_offset,
            constraints.widget_size,
            out_size,
            constraints.work_area,
            constraints.min_visible,
        );
        let reachable = out_size.is_some();
        let margins = margins_from_offset(anchor, clamped);

        PlacementDecision::Placed(PlacedSurface {
            target_output_id,
            anchor,
            offset: clamped,
            margins,
            reason: match_reason,
            is_fallback,
            is_reachable: reachable,
        })
    }

    /// Converts a completed user move interaction into a committed `MoveCommitResult`.
    ///
    /// Returns both the target `OutputId` and the persistable `SavedPlacement`.
    #[allow(clippy::too_many_arguments)]
    pub fn commit_move(
        pointer: (f64, f64),
        grab: (f64, f64),
        widget_size: (i32, i32),
        start_margins: (i32, i32, i32, i32),
        start_anchor: Anchor,
        current_output_id: OutputId,
        outputs: &[OutputSnapshot],
        work_area: Option<((i32, i32), (i32, i32))>,
        min_visible: i32,
    ) -> Option<MoveCommitResult> {
        let current_output = outputs.iter().find(|o| o.id == current_output_id)?;
        let current_size = current_output.logical_size?;
        let origin = origin_from_margins(start_anchor, start_margins, widget_size, current_size);

        let global = current_output.logical_position.map(|out_pos| {
            (
                out_pos.0 + origin.0 + pointer.0 as i32,
                out_pos.1 + origin.1 + pointer.1 as i32,
            )
        });

        let target = global
            .and_then(|point| containing_output(point, outputs))
            .cloned()
            .or_else(|| Some(current_output.clone()))?;

        let out_size = target.logical_size.unwrap_or(current_size);
        let local = if target.id == current_output_id {
            dragged_origin(origin, grab, pointer)
        } else {
            let pos = target.logical_position?;
            let glob = global?;
            (
                glob.0 - pos.0 - grab.0 as i32,
                glob.1 - pos.1 - grab.1 as i32,
            )
        };

        let (anchor, raw_offset) = nearest_corner(local, widget_size, out_size);
        let offset = clamp_offset(
            anchor,
            raw_offset,
            widget_size,
            Some(out_size),
            work_area,
            min_visible,
        );

        let target_output_id = target.id;
        let saved_placement = SavedPlacement {
            output: target.to_preference(),
            anchor,
            offset,
        };

        Some(MoveCommitResult {
            target_output_id,
            saved_placement,
            anchor,
            offset,
        })
    }
}

type FallbackOrderKey<'a> = (
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<(i32, i32)>,
    Option<(i32, i32)>,
    OutputId,
);

/// Fallback ordering key using stable properties first, OutputId only as a tie-breaker.
fn output_fallback_key(o: &OutputSnapshot) -> FallbackOrderKey<'_> {
    (
        o.name.as_deref(),
        o.serial(),
        o.identity.make.as_deref(),
        o.identity.model.as_deref(),
        o.identity.physical_size_mm,
        o.logical_position,
        o.id,
    )
}

fn fingerprint_matches(ident: &OutputIdentity, o: &OutputSnapshot) -> bool {
    let has_any = ident.make.is_some() || ident.model.is_some() || ident.physical_size_mm.is_some();
    if !has_any {
        return false;
    }
    ident.make.as_ref().map(|m| m == o.make()).unwrap_or(true)
        && ident.model.as_ref().map(|m| m == o.model()).unwrap_or(true)
        && ident
            .physical_size_mm
            .map(|p| p == o.physical_size_mm())
            .unwrap_or(true)
}

/// Clamps widget offset to stay within usable bounds, taking into account the semantic anchor,
/// output size, optional work area boundaries, and minimum visible handle size.
pub fn clamp_offset(
    anchor: Anchor,
    offset: (i32, i32),
    widget: (i32, i32),
    output: Option<(i32, i32)>,
    work_area: Option<((i32, i32), (i32, i32))>,
    min_visible: i32,
) -> (i32, i32) {
    let Some((ow, oh)) = output else {
        return offset;
    };

    let (reserved_left, reserved_top, reserved_right, reserved_bottom, avail_w, avail_h) =
        match work_area {
            Some(((wx, wy), (ww, wh))) => {
                let r_left = wx.clamp(0, ow);
                let r_top = wy.clamp(0, oh);
                let r_right = (ow - (wx + ww)).clamp(0, ow - r_left);
                let r_bottom = (oh - (wy + wh)).clamp(0, oh - r_top);
                let aw = (ow - r_left - r_right).max(0);
                let ah = (oh - r_top - r_bottom).max(0);
                (r_left, r_top, r_right, r_bottom, aw, ah)
            }
            None => (0, 0, 0, 0, ow, oh),
        };

    let w = widget.0.clamp(min_visible, ow.max(min_visible));
    let h = widget.1.clamp(min_visible, oh.max(min_visible));

    let max_span_x = (avail_w - w).max(0);
    let max_span_y = (avail_h - h).max(0);

    let clamped_x = match anchor {
        Anchor::TopRight | Anchor::BottomRight | Anchor::Right => {
            offset.0.clamp(reserved_right, reserved_right + max_span_x)
        }
        Anchor::TopLeft
        | Anchor::BottomLeft
        | Anchor::Left
        | Anchor::Top
        | Anchor::Bottom
        | Anchor::Center => offset.0.clamp(reserved_left, reserved_left + max_span_x),
    };

    let clamped_y = match anchor {
        Anchor::BottomLeft | Anchor::BottomRight | Anchor::Bottom => offset
            .1
            .clamp(reserved_bottom, reserved_bottom + max_span_y),
        Anchor::TopLeft
        | Anchor::TopRight
        | Anchor::Top
        | Anchor::Left
        | Anchor::Right
        | Anchor::Center => offset.1.clamp(reserved_top, reserved_top + max_span_y),
    };

    (clamped_x, clamped_y)
}

pub fn nearest_corner(
    origin: (i32, i32),
    widget: (i32, i32),
    output: (i32, i32),
) -> (Anchor, (i32, i32)) {
    let (x, y) = origin;
    let (ww, wh) = widget;
    let (ow, oh) = output;
    let cx = x + ww / 2;
    let cy = y + wh / 2;
    let d_tl = dist2(cx, cy, 0, 0);
    let d_tr = dist2(cx, cy, ow, 0);
    let d_bl = dist2(cx, cy, 0, oh);
    let d_br = dist2(cx, cy, ow, oh);
    let mut best = (d_tl, Anchor::TopLeft, (x, y));
    for cand in [
        (d_tr, Anchor::TopRight, (ow - x - ww, y)),
        (d_bl, Anchor::BottomLeft, (x, oh - y - wh)),
        (d_br, Anchor::BottomRight, (ow - x - ww, oh - y - wh)),
    ] {
        if cand.0 < best.0 {
            best = cand;
        }
    }
    (best.1, best.2)
}

fn dist2(x: i32, y: i32, ox: i32, oy: i32) -> i64 {
    let dx = (x - ox) as i64;
    let dy = (y - oy) as i64;
    dx * dx + dy * dy
}

pub fn apply_margin_delta(margins: &mut (i32, i32, i32, i32), anchor: Anchor, dx: i32, dy: i32) {
    match anchor {
        Anchor::TopLeft => {
            margins.3 += dx;
            margins.0 += dy;
        }
        Anchor::TopRight => {
            margins.1 -= dx;
            margins.0 += dy;
        }
        Anchor::BottomLeft => {
            margins.3 += dx;
            margins.2 -= dy;
        }
        Anchor::BottomRight => {
            margins.1 -= dx;
            margins.2 -= dy;
        }
        _ => {
            margins.3 += dx;
            margins.0 += dy;
        }
    }
}

pub fn origin_from_margins(
    anchor: Anchor,
    margins: (i32, i32, i32, i32),
    widget: (i32, i32),
    output: (i32, i32),
) -> (i32, i32) {
    let (top, right, bottom, left) = margins;
    let (ww, wh) = widget;
    let (ow, oh) = output;
    match anchor {
        Anchor::TopLeft => (left, top),
        Anchor::TopRight => (ow - ww - right, top),
        Anchor::BottomLeft => (left, oh - wh - bottom),
        Anchor::BottomRight => (ow - ww - right, oh - wh - bottom),
        Anchor::Top => (left, top),
        Anchor::Bottom => (left, oh - wh - bottom),
        Anchor::Left => (left, top),
        Anchor::Right => (ow - ww - right, top),
        Anchor::Center => (left, top),
    }
}

pub fn dragged_origin(origin: (i32, i32), grab: (f64, f64), pointer: (f64, f64)) -> (i32, i32) {
    (
        origin.0 + (pointer.0 - grab.0) as i32,
        origin.1 + (pointer.1 - grab.1) as i32,
    )
}

pub fn margins_from_offset(anchor: Anchor, offset: (i32, i32)) -> (i32, i32, i32, i32) {
    let (a, b) = offset;
    match anchor {
        Anchor::TopLeft => (b, 0, 0, a),
        Anchor::TopRight => (b, a, 0, 0),
        Anchor::BottomLeft => (0, 0, b, a),
        Anchor::BottomRight => (0, a, b, 0),
        Anchor::Top => (b, 0, 0, a),
        Anchor::Bottom => (0, 0, b, a),
        Anchor::Left => (b, 0, 0, a),
        Anchor::Right => (b, a, 0, 0),
        Anchor::Center => (b, 0, 0, a),
    }
}

pub fn containing_output(
    global: (i32, i32),
    outputs: &[OutputSnapshot],
) -> Option<&OutputSnapshot> {
    let mut matches = outputs.iter().filter(|o| {
        let Some((x, y)) = o.logical_position else {
            return false;
        };
        let Some((w, h)) = o.logical_size else {
            return false;
        };
        global.0 >= x && global.1 >= y && global.0 < x + w && global.1 < y + h
    });
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

impl From<&crate::domain::host::HostSpec> for HostPlacementConstraints {
    fn from(spec: &crate::domain::host::HostSpec) -> Self {
        let widget_size = match spec.size {
            crate::domain::host::HostSize::Exact { width, height } => (width as i32, height as i32),
            crate::domain::host::HostSize::OutputWidth { height } => (0, height as i32),
            crate::domain::host::HostSize::OutputHeight { width } => (width as i32, 0),
            crate::domain::host::HostSize::FillOutput => (0, 0),
            crate::domain::host::HostSize::ContentPreferred => (MIN_VISIBLE, MIN_VISIBLE),
        };

        Self {
            min_visible: MIN_VISIBLE,
            widget_size,
            work_area: None,
        }
    }
}
