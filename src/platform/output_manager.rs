use crate::domain::ids::OutputId;
use crate::domain::output::{OutputIdentity, OutputSnapshot};

/// Monotonically increasing generation token for committed output topology changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopologyGeneration(u64);

impl TopologyGeneration {
    pub const INITIAL: Self = Self(0);

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

impl std::fmt::Display for TopologyGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "topology_gen#{}", self.0)
    }
}

/// Attributes of a newly discovered output prior to runtime OutputId allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewOutput {
    pub name: Option<String>,
    pub identity: OutputIdentity,
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub logical_role: Option<String>,
    pub scale: i32,
    pub preferred: bool,
}

impl NewOutput {
    pub fn new(
        name: Option<String>,
        identity: OutputIdentity,
        logical_position: Option<(i32, i32)>,
        logical_size: Option<(i32, i32)>,
        logical_role: Option<String>,
        scale: i32,
        preferred: bool,
    ) -> Self {
        Self {
            name,
            identity,
            logical_position,
            logical_size,
            logical_role,
            scale,
            preferred,
        }
    }

    pub fn into_snapshot(self, id: OutputId) -> OutputSnapshot {
        OutputSnapshot {
            id,
            name: self.name,
            identity: self.identity,
            logical_position: self.logical_position,
            logical_size: self.logical_size,
            logical_role: self.logical_role,
            scale: self.scale,
            preferred: self.preferred,
        }
    }
}

/// Immutable, canonical snapshot of active output topology at a specific generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologySnapshot {
    generation: TopologyGeneration,
    outputs: Vec<OutputSnapshot>,
}

impl TopologySnapshot {
    pub fn new(generation: TopologyGeneration, outputs: Vec<OutputSnapshot>) -> Self {
        Self {
            generation,
            outputs,
        }
    }

    pub fn generation(&self) -> TopologyGeneration {
        self.generation
    }

    pub fn outputs(&self) -> &[OutputSnapshot] {
        &self.outputs
    }

    /// Returns an iterator over outputs that have valid logical geometry and are placement-capable.
    pub fn placement_capable_outputs(&self) -> impl Iterator<Item = &OutputSnapshot> {
        self.outputs.iter().filter(|o| o.logical_size.is_some())
    }

    pub fn get(&self, id: OutputId) -> Option<&OutputSnapshot> {
        self.outputs.iter().find(|o| o.id == id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&OutputSnapshot> {
        self.outputs
            .iter()
            .find(|o| o.name.as_deref() == Some(name))
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.outputs.len()
    }
}

/// Typed topology events emitted when output inventory or attributes change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyEvent {
    OutputAdded(OutputSnapshot),
    OutputUpdated(OutputSnapshot),
    OutputRemoved(OutputId),
    TopologyChanged(TopologySnapshot),
}

/// Pure state machine managing monotonic OutputId allocation, topology generations,
/// and canonical output inventory without raw Wayland dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRegistry {
    next_output_id: u32,
    generation: TopologyGeneration,
    active_outputs: Vec<OutputSnapshot>,
    retired_ids: Vec<OutputId>,
}

impl Default for OutputRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputRegistry {
    pub fn new() -> Self {
        Self {
            next_output_id: 1,
            generation: TopologyGeneration::INITIAL,
            active_outputs: Vec::new(),
            retired_ids: Vec::new(),
        }
    }

    pub fn generation(&self) -> TopologyGeneration {
        self.generation
    }

    pub fn snapshot(&self) -> TopologySnapshot {
        TopologySnapshot::new(self.generation, self.active_outputs.clone())
    }

    /// Exclusively allocates a new monotonic runtime `OutputId` and registers the new output.
    ///
    /// The allocated ID is guaranteed to be unique and never previously retired.
    pub fn add(&mut self, new_output: NewOutput) -> OutputId {
        let raw = self.next_output_id;
        self.next_output_id = self
            .next_output_id
            .checked_add(1)
            .expect("OutputId counter overflow");
        let id = OutputId::new(raw);
        assert!(
            !self.retired_ids.contains(&id),
            "OutputId reuse invariant violated"
        );
        let snapshot = new_output.into_snapshot(id);
        self.active_outputs.push(snapshot);
        self.generation = self.generation.next();
        id
    }

    /// Updates an existing active output snapshot.
    ///
    /// Returns `true` if the canonical inventory was changed (advancing topology generation),
    /// `false` if the output already matched existing state, or `false` if `snapshot.id` is not active / is retired.
    pub fn update(&mut self, snapshot: OutputSnapshot) -> bool {
        if self.retired_ids.contains(&snapshot.id) {
            return false;
        }
        let Some(existing) = self.active_outputs.iter_mut().find(|o| o.id == snapshot.id) else {
            return false;
        };
        if *existing == snapshot {
            return false;
        }
        *existing = snapshot;
        self.generation = self.generation.next();
        true
    }

    /// Synchronously evicts an output from the active canonical inventory.
    ///
    /// Invariant: Once `remove` returns, the removed output is immediately absent
    /// from `self.snapshot()` before consumers can reconcile against the removal.
    pub fn remove(&mut self, id: OutputId) -> Option<OutputSnapshot> {
        let pos = self.active_outputs.iter().position(|o| o.id == id)?;
        let removed = self.active_outputs.remove(pos);
        self.retired_ids.push(id);
        self.generation = self.generation.next();
        Some(removed)
    }

    pub fn is_active(&self, id: OutputId) -> bool {
        self.active_outputs.iter().any(|o| o.id == id)
    }

    pub fn is_retired(&self, id: OutputId) -> bool {
        self.retired_ids.contains(&id)
    }

    pub fn active_ids(&self) -> impl Iterator<Item = OutputId> + '_ {
        self.active_outputs.iter().map(|o| o.id)
    }

    pub fn get(&self, id: OutputId) -> Option<&OutputSnapshot> {
        self.active_outputs.iter().find(|o| o.id == id)
    }

    pub fn len(&self) -> usize {
        self.active_outputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_outputs.is_empty()
    }
}
