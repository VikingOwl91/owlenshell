use serde::Deserialize;
use std::cell::RefCell;
use std::process::Command;

/// Enriched output facts discovered from the active compositor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompositorOutputFacts {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub focused: bool,
}

/// Interface for compositor-specific metadata enrichment.
///
/// Compositor adapters return enrichment facts and preferred output context.
/// They do NOT own canonical output inventory or assign OutputIds.
pub trait CompositorAdapter {
    fn query_output_facts(&self) -> Vec<CompositorOutputFacts>;
    fn preferred_output_name(&self) -> Option<String>;
}

/// Fallback generic adapter when no supported compositor is detected.
#[derive(Debug, Default)]
pub struct GenericAdapter;

impl CompositorAdapter for GenericAdapter {
    fn query_output_facts(&self) -> Vec<CompositorOutputFacts> {
        Vec::new()
    }

    fn preferred_output_name(&self) -> Option<String> {
        None
    }
}

/// Hyprland compositor adapter querying `hyprctl -j monitors`.
pub struct HyprlandAdapter {
    initial_monitors: RefCell<Option<Vec<HyprMonitor>>>,
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

impl HyprlandAdapter {
    pub fn probe() -> Option<Self> {
        let monitors = Self::query()?;
        Some(Self {
            initial_monitors: RefCell::new(Some(monitors)),
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

impl CompositorAdapter for HyprlandAdapter {
    fn query_output_facts(&self) -> Vec<CompositorOutputFacts> {
        let monitors = if let Some(init) = self.initial_monitors.borrow_mut().take() {
            Some(init)
        } else {
            Self::query()
        };

        let Some(monitors) = monitors else {
            return Vec::new();
        };

        monitors
            .into_iter()
            .map(|m| CompositorOutputFacts {
                name: m.name,
                make: if m.make.is_empty() {
                    None
                } else {
                    Some(m.make)
                },
                model: if m.model.is_empty() {
                    None
                } else {
                    Some(m.model)
                },
                serial: if m.serial.is_empty() {
                    None
                } else {
                    Some(m.serial)
                },
                focused: m.focused,
            })
            .collect()
    }

    fn preferred_output_name(&self) -> Option<String> {
        let monitors = Self::query()?;
        monitors.into_iter().find(|m| m.focused).map(|m| m.name)
    }
}

/// Detects the active compositor environment and instantiates the appropriate adapter.
pub fn detect() -> Box<dyn CompositorAdapter> {
    let is_hyprland = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .is_ok_and(|desktop| desktop.to_ascii_lowercase().contains("hyprland"));
    if is_hyprland {
        if let Some(h) = HyprlandAdapter::probe() {
            tracing::info!("compositor adapter: hyprland");
            return Box::new(h);
        }
        tracing::warn!("hyprland detected but monitor query failed; using generic adapter");
    }
    tracing::info!("compositor adapter: generic");
    Box::new(GenericAdapter)
}
