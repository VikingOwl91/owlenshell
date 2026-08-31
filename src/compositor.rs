use crate::model::OutputSnapshot;
use serde::Deserialize;
use std::cell::{Cell, RefCell};
use std::process::Command;

pub trait CompositorBackend {
    fn enrich_outputs(&self, outputs: &mut [OutputSnapshot]);
    fn preferred_output(&self) -> Option<String>;
}

pub struct GenericBackend;

impl CompositorBackend for GenericBackend {
    fn enrich_outputs(&self, _outputs: &mut [OutputSnapshot]) {}
    fn preferred_output(&self) -> Option<String> {
        None
    }
}

pub struct HyprlandBackend {
    monitors: RefCell<Vec<HyprMonitor>>,
    skip_next_refresh: Cell<bool>,
}

#[derive(Debug, Deserialize)]
struct HyprMonitor {
    name: String,
    #[serde(default)]
    make: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    serial: String,
    #[serde(default)]
    focused: bool,
}

impl HyprlandBackend {
    pub fn probe() -> Option<Self> {
        let monitors = Self::query()?;
        Some(Self {
            monitors: RefCell::new(monitors),
            skip_next_refresh: Cell::new(true),
        })
    }

    fn query() -> Option<Vec<HyprMonitor>> {
        let out = Command::new("hyprctl")
            .args(["-j", "monitors"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        serde_json::from_slice(&out.stdout).ok()
    }
}

impl CompositorBackend for HyprlandBackend {
    fn enrich_outputs(&self, outputs: &mut [OutputSnapshot]) {
        if self.skip_next_refresh.replace(false) {
            // probe() already captured the topology used for first placement.
        } else if let Some(monitors) = Self::query() {
            *self.monitors.borrow_mut() = monitors;
        }
        let monitors = self.monitors.borrow();
        for o in outputs.iter_mut() {
            let Some(name) = o.name.as_deref() else {
                continue;
            };
            let Some(m) = monitors.iter().find(|m| m.name == name) else {
                continue;
            };
            if !m.serial.is_empty() {
                o.serial = Some(m.serial.clone());
            }
            if !m.make.is_empty() && o.make.is_empty() {
                o.make = m.make.clone();
            }
            if !m.model.is_empty() && o.model.is_empty() {
                o.model = m.model.clone();
            }
            if m.focused {
                o.preferred = true;
            }
        }
    }

    fn preferred_output(&self) -> Option<String> {
        self.monitors
            .borrow()
            .iter()
            .find(|m| m.focused)
            .map(|m| m.name.clone())
    }
}

pub fn detect() -> Box<dyn CompositorBackend> {
    let is_hyprland = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .is_ok_and(|desktop| desktop.to_ascii_lowercase().contains("hyprland"));
    if is_hyprland {
        if let Some(h) = HyprlandBackend::probe() {
            tracing::info!("compositor adapter: hyprland");
            return Box::new(h);
        }
        tracing::warn!("hyprland detected but monitor query failed; using generic adapter");
    }
    {
        tracing::info!("compositor adapter: generic");
        Box::new(GenericBackend)
    }
}
