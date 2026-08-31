use crate::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use crate::domain::ids::HostId;
use crate::domain::placement::PlacedSurface;
use std::collections::BTreeMap;

/// Monotonically increasing generation token for native surface instances of a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceGeneration(u64);

impl SurfaceGeneration {
    pub const INITIAL: Self = Self(1);

    pub fn new(val: u64) -> Self {
        Self(val)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::fmt::Display for SurfaceGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "surface_gen#{}", self.0)
    }
}

/// Ephemeral runtime token identifying a specific native instance of a host surface.
///
/// Captures both host identity and instance generation for stale callback rejection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SurfaceToken {
    pub host_id: HostId,
    pub generation: SurfaceGeneration,
}

impl SurfaceToken {
    pub fn new(host_id: HostId, generation: SurfaceGeneration) -> Self {
        Self {
            host_id,
            generation,
        }
    }

    pub fn host_id(&self) -> &HostId {
        &self.host_id
    }

    pub fn generation(&self) -> SurfaceGeneration {
        self.generation
    }
}

impl std::fmt::Display for SurfaceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "token({}:{})", self.host_id, self.generation)
    }
}

/// State slot tracking generation lifecycle for a single host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSlot {
    pub host_id: HostId,
    pub next_generation: SurfaceGeneration,
    pub current_generation: Option<SurfaceGeneration>,
}

impl SurfaceSlot {
    pub fn new(host_id: HostId) -> Self {
        Self {
            host_id,
            next_generation: SurfaceGeneration::INITIAL,
            current_generation: None,
        }
    }
}

/// Pure state machine managing monotonic generation allocation, active token validation,
/// and stale callback rejection without raw Wayland/SCTK dependencies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfaceGenerationRegistry {
    slots: BTreeMap<HostId, SurfaceSlot>,
}

impl SurfaceGenerationRegistry {
    pub fn new() -> Self {
        Self {
            slots: BTreeMap::new(),
        }
    }

    /// Allocates an initial or new native instance token for `host_id`.
    pub fn allocate(&mut self, host_id: HostId) -> SurfaceToken {
        let slot = self
            .slots
            .entry(host_id.clone())
            .or_insert_with(|| SurfaceSlot::new(host_id.clone()));
        let gen = slot.next_generation;
        slot.next_generation = gen.next();
        slot.current_generation = Some(gen);
        SurfaceToken::new(host_id, gen)
    }

    /// Recreates the native instance for `host_id`, retiring the previous generation
    /// and allocating a strictly monotonic incremented generation.
    pub fn recreate(&mut self, host_id: &HostId) -> SurfaceToken {
        let slot = self
            .slots
            .entry(host_id.clone())
            .or_insert_with(|| SurfaceSlot::new(host_id.clone()));
        let gen = slot.next_generation;
        slot.next_generation = gen.next();
        slot.current_generation = Some(gen);
        SurfaceToken::new(host_id.clone(), gen)
    }

    /// Retires the currently active generation for `host_id`.
    ///
    /// Invariant: Does NOT reset `next_generation`. Later allocations continue monotonically.
    pub fn retire(&mut self, host_id: &HostId) -> Option<SurfaceToken> {
        let slot = self.slots.get_mut(host_id)?;
        let gen = slot.current_generation.take()?;
        Some(SurfaceToken::new(host_id.clone(), gen))
    }

    /// Hides/destroys the native instance, retiring the token while preserving generation state.
    pub fn hide(&mut self, host_id: &HostId) -> Option<SurfaceToken> {
        self.retire(host_id)
    }

    /// Validates whether a token represents the currently active generation for an expected host.
    ///
    /// Returns `false` if:
    /// - `token.host_id` does not match `expected_host_id` (cross-host forgery)
    /// - `expected_host_id` is unknown or currently has no active generation
    /// - `token.generation` does not match the active current generation (stale callback or retired)
    pub fn validate(&self, expected_host_id: &HostId, token: &SurfaceToken) -> bool {
        if token.host_id != *expected_host_id {
            return false;
        }
        if let Some(slot) = self.slots.get(expected_host_id) {
            slot.current_generation == Some(token.generation)
        } else {
            false
        }
    }

    pub fn is_current(&self, host_id: &HostId, generation: SurfaceGeneration) -> bool {
        if let Some(slot) = self.slots.get(host_id) {
            slot.current_generation == Some(generation)
        } else {
            false
        }
    }

    pub fn current_token(&self, host_id: &HostId) -> Option<SurfaceToken> {
        let slot = self.slots.get(host_id)?;
        let gen = slot.current_generation?;
        Some(SurfaceToken::new(host_id.clone(), gen))
    }

    pub fn current_generation(&self, host_id: &HostId) -> Option<SurfaceGeneration> {
        self.slots.get(host_id).and_then(|s| s.current_generation)
    }

    pub fn has_active_surface(&self, host_id: &HostId) -> bool {
        self.slots
            .get(host_id)
            .map(|s| s.current_generation.is_some())
            .unwrap_or(false)
    }
}

/// Generic native surface specification provided to SurfaceManager.
///
/// Contains NO widget-specific or calendar/bar variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSpec {
    pub namespace: String,
    pub layer: HostLayer,
    pub mode: HostMode,
    pub size: HostSize,
    pub exclusive_zone: ExclusiveZonePolicy,
    pub keyboard_interactivity: KeyboardInteractivityPolicy,
}

/// Typed lifecycle commands sent from runtime coordinator to SurfaceManager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceCommand {
    EnsureVisible {
        host_id: HostId,
        spec: SurfaceSpec,
        placement: PlacedSurface,
    },
    Reconfigure {
        host_id: HostId,
        spec: SurfaceSpec,
        placement: PlacedSurface,
    },
    RecreateOnOutput {
        host_id: HostId,
        spec: SurfaceSpec,
        placement: PlacedSurface,
    },
    Hide {
        host_id: HostId,
    },
    Destroy {
        host_id: HostId,
    },
    RequestFrame {
        host_id: HostId,
    },
}

/// Typed surface lifecycle event emitted by platform to coordinator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceLifecycleEvent {
    Configured {
        token: SurfaceToken,
        width: u32,
        height: u32,
    },
    FrameRendered {
        token: SurfaceToken,
    },
    Closed {
        token: SurfaceToken,
    },
}
