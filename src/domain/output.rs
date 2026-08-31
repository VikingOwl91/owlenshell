use crate::domain::ids::OutputId;

/// Stable matching evidence for physical monitor identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct OutputIdentity {
    pub make: Option<String>,
    pub model: Option<String>,
    pub physical_size_mm: Option<(i32, i32)>,
    pub serial: Option<String>,
}

impl OutputIdentity {
    pub fn is_empty(&self) -> bool {
        self.make.is_none()
            && self.model.is_none()
            && self.physical_size_mm.is_none()
            && self.serial.is_none()
    }
}

/// Persisted user output placement preference.
///
/// Contains matching evidence (connector name, physical hardware identity, logical role).
/// Never contains transient runtime OutputId.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct OutputPreference {
    pub name: Option<String>,
    pub identity: Option<OutputIdentity>,
    pub logical_role: Option<String>,
}

impl OutputPreference {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        let identity = self.identity.get_or_insert_with(Default::default);
        identity.serial = Some(serial.into());
        self
    }

    pub fn with_make(mut self, make: impl Into<String>) -> Self {
        let identity = self.identity.get_or_insert_with(Default::default);
        identity.make = Some(make.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let identity = self.identity.get_or_insert_with(Default::default);
        identity.model = Some(model.into());
        self
    }

    pub fn with_physical_size_mm(mut self, size: (i32, i32)) -> Self {
        let identity = self.identity.get_or_insert_with(Default::default);
        identity.physical_size_mm = Some(size);
        self
    }

    pub fn with_logical_role(mut self, role: impl Into<String>) -> Self {
        self.logical_role = Some(role.into());
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn serial(&self) -> Option<&str> {
        self.identity.as_ref().and_then(|i| i.serial.as_deref())
    }

    pub fn make(&self) -> Option<&str> {
        self.identity.as_ref().and_then(|i| i.make.as_deref())
    }

    pub fn model(&self) -> Option<&str> {
        self.identity.as_ref().and_then(|i| i.model.as_deref())
    }

    pub fn physical_size_mm(&self) -> Option<(i32, i32)> {
        self.identity.as_ref().and_then(|i| i.physical_size_mm)
    }
}

/// Current runtime snapshot of an advertised Wayland output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSnapshot {
    pub id: OutputId,
    pub name: Option<String>,
    pub identity: OutputIdentity,
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub logical_role: Option<String>,
    pub scale: i32,
    pub preferred: bool,
}

impl OutputSnapshot {
    pub fn to_preference(&self) -> OutputPreference {
        OutputPreference {
            name: self.name.clone(),
            identity: if self.identity.is_empty() {
                None
            } else {
                Some(self.identity.clone())
            },
            logical_role: self.logical_role.clone(),
        }
    }

    pub fn make(&self) -> &str {
        self.identity.make.as_deref().unwrap_or("")
    }

    pub fn model(&self) -> &str {
        self.identity.model.as_deref().unwrap_or("")
    }

    pub fn serial(&self) -> Option<&str> {
        self.identity.serial.as_deref()
    }

    pub fn physical_size_mm(&self) -> (i32, i32) {
        self.identity.physical_size_mm.unwrap_or((0, 0))
    }

    /// Returns true if the output has positive logical dimensions suitable for surface placement.
    pub fn is_placement_capable(&self) -> bool {
        matches!(self.logical_size, Some((w, h)) if w > 0 && h > 0)
    }
}
