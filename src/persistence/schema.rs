use crate::domain::output::{OutputIdentity, OutputPreference};
use crate::domain::placement::{Anchor, SavedPlacement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Persisted anchor representation DTO.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorDto {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl From<Anchor> for AnchorDto {
    fn from(a: Anchor) -> Self {
        match a {
            Anchor::TopLeft => Self::TopLeft,
            Anchor::TopRight => Self::TopRight,
            Anchor::BottomLeft => Self::BottomLeft,
            Anchor::BottomRight => Self::BottomRight,
            Anchor::Top => Self::Top,
            Anchor::Bottom => Self::Bottom,
            Anchor::Left => Self::Left,
            Anchor::Right => Self::Right,
            Anchor::Center => Self::Center,
        }
    }
}

impl From<AnchorDto> for Anchor {
    fn from(dto: AnchorDto) -> Self {
        match dto {
            AnchorDto::TopLeft => Self::TopLeft,
            AnchorDto::TopRight => Self::TopRight,
            AnchorDto::BottomLeft => Self::BottomLeft,
            AnchorDto::BottomRight => Self::BottomRight,
            AnchorDto::Top => Self::Top,
            AnchorDto::Bottom => Self::Bottom,
            AnchorDto::Left => Self::Left,
            AnchorDto::Right => Self::Right,
            AnchorDto::Center => Self::Center,
        }
    }
}

/// Persisted output matching evidence schema DTO.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputIdentityDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_size_mm: Option<(i32, i32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
}

impl From<&OutputIdentity> for OutputIdentityDto {
    fn from(id: &OutputIdentity) -> Self {
        Self {
            make: id.make.clone(),
            model: id.model.clone(),
            physical_size_mm: id.physical_size_mm,
            serial: id.serial.clone(),
        }
    }
}

impl From<OutputIdentityDto> for OutputIdentity {
    fn from(dto: OutputIdentityDto) -> Self {
        Self {
            make: dto.make,
            model: dto.model,
            physical_size_mm: dto.physical_size_mm,
            serial: dto.serial,
        }
    }
}

/// Persisted output preference schema DTO.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPreferenceDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<OutputIdentityDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_role: Option<String>,
}

impl From<&OutputPreference> for OutputPreferenceDto {
    fn from(pref: &OutputPreference) -> Self {
        Self {
            name: pref.name.clone(),
            identity: pref.identity.as_ref().map(OutputIdentityDto::from),
            logical_role: pref.logical_role.clone(),
        }
    }
}

impl From<OutputPreferenceDto> for OutputPreference {
    fn from(dto: OutputPreferenceDto) -> Self {
        Self {
            name: dto.name,
            identity: dto.identity.map(OutputIdentity::from),
            logical_role: dto.logical_role,
        }
    }
}

/// Persisted placement entry schema DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPlacementDto {
    pub output: OutputPreferenceDto,
    pub anchor: AnchorDto,
    pub offset: (i32, i32),
}

impl From<&SavedPlacement> for SavedPlacementDto {
    fn from(domain: &SavedPlacement) -> Self {
        Self {
            output: OutputPreferenceDto::from(&domain.output),
            anchor: AnchorDto::from(domain.anchor),
            offset: domain.offset,
        }
    }
}

impl From<SavedPlacementDto> for SavedPlacement {
    fn from(dto: SavedPlacementDto) -> Self {
        Self {
            output: OutputPreference::from(dto.output),
            anchor: Anchor::from(dto.anchor),
            offset: dto.offset,
        }
    }
}

/// Versioned persistent file root schema DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementsFileDto {
    pub version: u32,
    #[serde(default)]
    pub widgets: BTreeMap<String, SavedPlacementDto>,
}

impl Default for PlacementsFileDto {
    fn default() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            widgets: BTreeMap::new(),
        }
    }
}
