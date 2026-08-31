use super::schema::{PlacementsFileDto, SavedPlacementDto, CURRENT_SCHEMA_VERSION};
use crate::domain::ids::HostId;
use crate::domain::placement::SavedPlacement;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SAVE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Runtime domain collection of saved host/widget placements.
///
/// Encapsulates version and indexed host placements with typed HostId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placements {
    version: u32,
    widgets: BTreeMap<HostId, SavedPlacement>,
}

impl Default for Placements {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            widgets: BTreeMap::new(),
        }
    }
}

/// Explicit outcome of loading persistence data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    /// Successfully loaded valid current-version placements.
    Loaded(Placements),
    /// Persistence file was not found; returning clean default.
    DefaultEmpty,
    /// Persistence file specified an unsupported schema version; returned default.
    UnsupportedVersion { version: u32, fallback: Placements },
    /// Persistence file was corrupt; backup file was verified on disk and returned default.
    CorruptedPreserved {
        backup_path: PathBuf,
        error: String,
        fallback: Placements,
    },
    /// Persistence file was corrupt, but backup preservation failed.
    PreservationFailed {
        backup_path: PathBuf,
        preserve_error: String,
        parse_error: String,
        fallback: Placements,
    },
    /// Reading the file failed due to I/O or permissions (not NotFound).
    ReadFailed { error: String, fallback: Placements },
}

impl Placements {
    /// Returns the schema version of this placements instance.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns default machine-local state path.
    pub fn path() -> PathBuf {
        let base = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
                PathBuf::from(home).join(".local/state")
            });
        base.join("owlenshell/placements.toml")
    }

    /// Loads placements from default path.
    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    /// Loads placements from a specified path, returning fallback on error.
    pub fn load_from(path: &Path) -> Self {
        match Self::load_with_outcome(path) {
            LoadOutcome::Loaded(p) => p,
            LoadOutcome::DefaultEmpty => Self::default(),
            LoadOutcome::UnsupportedVersion { fallback, .. } => fallback,
            LoadOutcome::CorruptedPreserved { fallback, .. } => fallback,
            LoadOutcome::PreservationFailed { fallback, .. } => fallback,
            LoadOutcome::ReadFailed { fallback, .. } => fallback,
        }
    }

    /// Loads placements with detailed outcome information for observability and testing.
    pub fn load_with_outcome(path: &Path) -> LoadOutcome {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return LoadOutcome::DefaultEmpty;
            }
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "failed to read placements file");
                return LoadOutcome::ReadFailed {
                    error: err.to_string(),
                    fallback: Self::default(),
                };
            }
        };

        let file_dto: Result<PlacementsFileDto, _> = toml::from_str(&content);
        match file_dto {
            Ok(dto) if dto.version == CURRENT_SCHEMA_VERSION => {
                let widgets = dto
                    .widgets
                    .into_iter()
                    .map(|(k, v)| (HostId::new(k), SavedPlacement::from(v)))
                    .collect();
                LoadOutcome::Loaded(Placements {
                    version: dto.version,
                    widgets,
                })
            }
            Ok(dto) => {
                tracing::warn!(
                    version = dto.version,
                    path = %path.display(),
                    "unsupported placements version"
                );
                LoadOutcome::UnsupportedVersion {
                    version: dto.version,
                    fallback: Self::default(),
                }
            }
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "corrupted placements file; preserving backup");
                Self::preserve_corrupt(path, &err.to_string(), &content)
            }
        }
    }

    fn preserve_corrupt(path: &Path, error_msg: &str, content: &str) -> LoadOutcome {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let counter = SAVE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let backup_path = path.with_extension(format!("corrupted.{ts}_{pid}_{counter}.bak"));

        let write_res = if !content.is_empty() {
            std::fs::write(&backup_path, content)
        } else if path.exists() {
            std::fs::copy(path, &backup_path).map(|_| ())
        } else {
            return LoadOutcome::PreservationFailed {
                backup_path,
                preserve_error: "source file does not exist to preserve".into(),
                parse_error: error_msg.to_string(),
                fallback: Self::default(),
            };
        };

        match write_res {
            Ok(()) if backup_path.exists() => LoadOutcome::CorruptedPreserved {
                backup_path,
                error: error_msg.to_string(),
                fallback: Self::default(),
            },
            Ok(()) => LoadOutcome::PreservationFailed {
                backup_path,
                preserve_error: "backup file was not found on disk after write".into(),
                parse_error: error_msg.to_string(),
                fallback: Self::default(),
            },
            Err(err) => LoadOutcome::PreservationFailed {
                backup_path,
                preserve_error: err.to_string(),
                parse_error: error_msg.to_string(),
                fallback: Self::default(),
            },
        }
    }

    /// Atomically persists placements to default path.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&Self::path())
    }

    /// Atomically persists placements to specified path via temporary write and rename.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let pid = std::process::id();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let counter = SAVE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_extension(format!("toml.tmp.{pid}_{ts}_{counter}"));
        let dto = PlacementsFileDto {
            version: self.version,
            widgets: self
                .widgets
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), SavedPlacementDto::from(v)))
                .collect(),
        };
        let body = toml::to_string_pretty(&dto).map_err(std::io::Error::other)?;
        if let Err(err) = std::fs::write(&tmp, body) {
            let _ = std::fs::remove_file(&tmp);
            return Err(err);
        }
        if let Err(err) = std::fs::rename(&tmp, path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(err);
        }
        Ok(())
    }

    /// Primary production API: Get placement by typed HostId.
    pub fn get(&self, host_id: &HostId) -> Option<&SavedPlacement> {
        self.widgets.get(host_id)
    }

    /// Primary production API: Set placement by typed HostId.
    pub fn set(&mut self, host_id: HostId, placement: SavedPlacement) {
        self.widgets.insert(host_id, placement);
    }

    /// Primary production API: Remove placement by typed HostId.
    pub fn remove(&mut self, host_id: &HostId) -> Option<SavedPlacement> {
        self.widgets.remove(host_id)
    }

    /// Returns true if no placements are stored.
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    /// Returns the count of stored host placements.
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Iterates over all stored host placements.
    pub fn iter(&self) -> impl Iterator<Item = (&HostId, &SavedPlacement)> {
        self.widgets.iter()
    }

    /// Compatibility helper for string-keyed lookups.
    pub fn get_str(&self, key: &str) -> Option<&SavedPlacement> {
        self.widgets.get(&HostId::new(key))
    }

    /// Compatibility helper for string-keyed insertions.
    pub fn set_str(&mut self, key: &str, placement: SavedPlacement) {
        self.widgets.insert(HostId::new(key), placement);
    }
}
