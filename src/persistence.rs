use crate::model::SavedPlacement;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placements {
    pub version: u32,
    #[serde(default)]
    pub widgets: BTreeMap<String, SavedPlacement>,
}

impl Default for Placements {
    fn default() -> Self {
        Self {
            version: 1,
            widgets: BTreeMap::new(),
        }
    }
}

impl Placements {
    pub fn path() -> PathBuf {
        let base = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
                PathBuf::from(home).join(".local/state")
            });
        base.join("owlenshell/placements.toml")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    pub fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(s) => match toml::from_str::<Self>(&s) {
                Ok(placements) if placements.version == 1 => placements,
                Ok(placements) => {
                    tracing::warn!(
                        version = placements.version,
                        path = %path.display(),
                        "unsupported placements version"
                    );
                    Self::default()
                }
                Err(err) => {
                    tracing::warn!(%err, path = %path.display(), "failed to parse placements");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&Self::path())
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension(format!("toml.tmp.{}", std::process::id()));
        let body = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&SavedPlacement> {
        self.widgets.get(key)
    }

    pub fn set(&mut self, key: &str, placement: SavedPlacement) {
        self.widgets.insert(key.to_string(), placement);
    }
}
