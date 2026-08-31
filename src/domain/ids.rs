use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HostId(String);

impl HostId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HostId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for HostId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for HostId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::ops::Deref for HostId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for HostId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WidgetId(String);

impl WidgetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for WidgetId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for WidgetId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::ops::Deref for WidgetId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for WidgetId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Runtime-only output identifier.
///
/// Note: This type intentionally DOES NOT implement `Serialize` or `Deserialize`.
/// Runtime `OutputId` must never be persisted as physical monitor identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId(pub u32);

impl OutputId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for OutputId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "output#{}", self.0)
    }
}

impl From<u32> for OutputId {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<OutputId> for u32 {
    fn from(id: OutputId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeatId(String);

impl SeatId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SeatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SeatId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SeatId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::ops::Deref for SeatId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SeatId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
