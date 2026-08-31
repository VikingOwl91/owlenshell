use crate::model::{Anchor, OutputPreference, OutputSnapshot, MIN_VISIBLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveReason {
    ExactName,
    Serial,
    Fingerprint,
    SpawnOutput,
    Preferred,
    Any,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    pub output_id: Option<u32>,
    pub reason: ResolveReason,
}

#[derive(Debug, Clone, Default)]
pub struct ResolveContext {
    pub spawn_output_id: Option<u32>,
    pub preferred_name: Option<String>,
}

pub fn resolve(
    pref: &OutputPreference,
    outputs: &[OutputSnapshot],
    ctx: &ResolveContext,
) -> ResolveResult {
    if outputs.is_empty() {
        return ResolveResult {
            output_id: None,
            reason: ResolveReason::Deferred,
        };
    }

    if let Some(name) = pref.name.as_deref() {
        if let Some(o) = outputs.iter().find(|o| o.name.as_deref() == Some(name)) {
            return ResolveResult {
                output_id: Some(o.id),
                reason: ResolveReason::ExactName,
            };
        }
    }

    if let Some(serial) = pref.serial.as_deref().filter(|s| !s.is_empty()) {
        let matches: Vec<&OutputSnapshot> = outputs
            .iter()
            .filter(|o| o.serial.as_deref() == Some(serial))
            .collect();
        if matches.len() == 1 {
            return ResolveResult {
                output_id: Some(matches[0].id),
                reason: ResolveReason::Serial,
            };
        }
    }

    if pref.make.is_some() || pref.model.is_some() || pref.physical_size_mm.is_some() {
        let matches: Vec<&OutputSnapshot> = outputs
            .iter()
            .filter(|o| fingerprint_matches(pref, o))
            .collect();
        if matches.len() == 1 {
            return ResolveResult {
                output_id: Some(matches[0].id),
                reason: ResolveReason::Fingerprint,
            };
        }
    }

    if let Some(id) = ctx.spawn_output_id {
        if outputs.iter().any(|o| o.id == id) {
            return ResolveResult {
                output_id: Some(id),
                reason: ResolveReason::SpawnOutput,
            };
        }
    }

    if let Some(name) = ctx.preferred_name.as_deref() {
        if let Some(o) = outputs.iter().find(|o| o.name.as_deref() == Some(name)) {
            return ResolveResult {
                output_id: Some(o.id),
                reason: ResolveReason::Preferred,
            };
        }
    }

    if let Some(o) = outputs.iter().find(|o| o.preferred) {
        return ResolveResult {
            output_id: Some(o.id),
            reason: ResolveReason::Preferred,
        };
    }

    let min = outputs.iter().min_by_key(|o| o.id).unwrap();
    ResolveResult {
        output_id: Some(min.id),
        reason: ResolveReason::Any,
    }
}

fn fingerprint_matches(pref: &OutputPreference, o: &OutputSnapshot) -> bool {
    pref.make.as_ref().map(|m| m == &o.make).unwrap_or(true)
        && pref.model.as_ref().map(|m| m == &o.model).unwrap_or(true)
        && pref
            .physical_size_mm
            .map(|p| p == o.physical_size_mm)
            .unwrap_or(true)
}

pub fn clamp_offset(
    offset: (i32, i32),
    widget: (i32, i32),
    output: Option<(i32, i32)>,
) -> (i32, i32) {
    let Some((ow, oh)) = output else {
        return offset;
    };
    let w = widget.0.clamp(MIN_VISIBLE, ow.max(MIN_VISIBLE));
    let h = widget.1.clamp(MIN_VISIBLE, oh.max(MIN_VISIBLE));
    let max_x = (ow - w).max(0);
    let max_y = (oh - h).max(0);
    (offset.0.clamp(0, max_x), offset.1.clamp(0, max_y))
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
