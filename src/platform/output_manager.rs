use crate::domain::ids::OutputId;
use crate::domain::output::{OutputIdentity, OutputSnapshot};
use crate::platform::compositor::{CompositorAdapter, CompositorOutputFacts};
use smithay_client_toolkit::output::OutputState;
use std::collections::HashMap;
use wayland_client::protocol::wl_output::WlOutput;

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

/// Raw protocol output facts from the display server prior to platform ingestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutput {
    pub native_id: u32,
    pub name: Option<String>,
    pub make: String,
    pub model: String,
    pub physical_size: (i32, i32),
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub scale_factor: i32,
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

/// Deterministic ordering for canonical output snapshots.
///
/// Ensures equivalent topology discovered in different protocol orders produces
/// the exact same canonical snapshot ordering.
pub fn canonical_output_cmp(a: &OutputSnapshot, b: &OutputSnapshot) -> std::cmp::Ordering {
    a.logical_position
        .cmp(&b.logical_position)
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.identity.make.cmp(&b.identity.make))
        .then_with(|| a.identity.model.cmp(&b.identity.model))
        .then_with(|| a.identity.serial.cmp(&b.identity.serial))
        .then_with(|| a.id.cmp(&b.id))
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
    ///
    /// Requires `logical_size` to be `Some((w, h))` with `w > 0` and `h > 0`.
    pub fn placement_capable_outputs(&self) -> impl Iterator<Item = &OutputSnapshot> {
        self.outputs.iter().filter(|o| o.is_placement_capable())
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
/// deterministic ordering, and canonical output inventory.
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

    /// Allocates a new monotonic runtime `OutputId`.
    pub fn allocate_id(&mut self) -> OutputId {
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
        id
    }

    /// Retires an OutputId so it can never be reused.
    pub fn retire_id(&mut self, id: OutputId) {
        if !self.retired_ids.contains(&id) {
            self.retired_ids.push(id);
        }
    }

    /// Atomically commits a new canonical topology batch.
    ///
    /// - Sorts the candidate snapshots deterministically.
    /// - If the new active inventory matches the current inventory, generation is unchanged and returns `None`.
    /// - If changed, any removed IDs are retired, generation advances exactly once, and the new `TopologySnapshot` is returned.
    pub fn commit_topology(
        &mut self,
        mut new_active: Vec<OutputSnapshot>,
    ) -> Option<TopologySnapshot> {
        new_active.sort_by(canonical_output_cmp);

        if self.active_outputs == new_active {
            return None;
        }

        // Retire any IDs that were active but are no longer present
        let removed_ids: Vec<OutputId> = self
            .active_outputs
            .iter()
            .filter_map(|old| (!new_active.iter().any(|n| n.id == old.id)).then_some(old.id))
            .collect();

        for id in removed_ids {
            self.retire_id(id);
        }

        self.active_outputs = new_active;
        self.generation = self.generation.next();
        Some(self.snapshot())
    }

    /// Exclusively allocates a new monotonic runtime `OutputId` and registers the new output as a single commit.
    pub fn add(&mut self, new_output: NewOutput) -> OutputId {
        let id = self.allocate_id();
        let snapshot = new_output.into_snapshot(id);
        let mut next_outputs = self.active_outputs.clone();
        next_outputs.push(snapshot);
        self.commit_topology(next_outputs);
        id
    }

    /// Updates an existing active output snapshot as a single commit.
    pub fn update(&mut self, snapshot: OutputSnapshot) -> bool {
        if self.retired_ids.contains(&snapshot.id) {
            return false;
        }
        let Some(pos) = self.active_outputs.iter().position(|o| o.id == snapshot.id) else {
            return false;
        };
        if self.active_outputs[pos] == snapshot {
            return false;
        }
        let mut next_outputs = self.active_outputs.clone();
        next_outputs[pos] = snapshot;
        self.commit_topology(next_outputs).is_some()
    }

    /// Synchronously evicts an output from active inventory.
    ///
    /// Returns the removed snapshot and the new committed topology snapshot, or `None` if not active.
    pub fn remove(&mut self, id: OutputId) -> Option<(OutputSnapshot, TopologySnapshot)> {
        let pos = self.active_outputs.iter().position(|o| o.id == id)?;
        let removed = self.active_outputs.remove(pos);
        self.retire_id(id);
        self.generation = self.generation.next();
        Some((removed, self.snapshot()))
    }

    pub fn placement_capable_outputs(&self) -> impl Iterator<Item = &OutputSnapshot> {
        self.active_outputs
            .iter()
            .filter(|o| o.is_placement_capable())
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

/// Helper to find an unambiguous compositor enrichment fact for a connector name.
///
/// If multiple facts are returned for the same connector name, the connector is ambiguous
/// and enrichment is skipped rather than choosing an arbitrary match.
fn find_unambiguous_fact<'a>(
    facts: &'a [CompositorOutputFacts],
    name: &str,
) -> Option<&'a CompositorOutputFacts> {
    let mut matches = facts.iter().filter(|f| f.name == name);
    let first = matches.next()?;
    if matches.next().is_some() {
        // Ambiguous duplicate facts for this connector name
        None
    } else {
        Some(first)
    }
}

/// OutputManager owns the live output inventory, native Wayland mappings,
/// compositor adapter enrichment, and canonical topology generation.
pub struct OutputManager {
    registry: OutputRegistry,
    native_by_id: HashMap<OutputId, WlOutput>,
    id_by_native_id: HashMap<u32, OutputId>,
    adapter: Box<dyn CompositorAdapter>,
}

impl OutputManager {
    pub fn new(adapter: Box<dyn CompositorAdapter>) -> Self {
        Self {
            registry: OutputRegistry::new(),
            native_by_id: HashMap::new(),
            id_by_native_id: HashMap::new(),
            adapter,
        }
    }

    pub fn snapshot(&self) -> TopologySnapshot {
        self.registry.snapshot()
    }

    pub fn generation(&self) -> TopologyGeneration {
        self.registry.generation()
    }

    pub fn placement_capable_outputs(&self) -> impl Iterator<Item = &OutputSnapshot> {
        self.registry.placement_capable_outputs()
    }

    pub fn get(&self, id: OutputId) -> Option<&OutputSnapshot> {
        self.registry.get(id)
    }

    pub fn get_wl_output(&self, id: OutputId) -> Option<WlOutput> {
        self.native_by_id.get(&id).cloned()
    }

    pub fn find_id_by_wl_output(&self, wl_output: &WlOutput) -> Option<OutputId> {
        self.native_by_id
            .iter()
            .find_map(|(id, candidate)| (candidate == wl_output).then_some(*id))
    }

    pub fn find_id_by_native_id(&self, native_id: u32) -> Option<OutputId> {
        self.id_by_native_id.get(&native_id).copied()
    }

    pub fn preferred_output_name(&self) -> Option<String> {
        self.adapter.preferred_output_name()
    }

    /// Ingests raw protocol outputs and fresh compositor enrichment in a single atomic transaction.
    ///
    /// Returns `Some(TopologySnapshot)` if canonical topology changed, or `None` if unchanged.
    pub fn sync_protocol_facts(
        &mut self,
        protocol_outputs: &[ProtocolOutput],
    ) -> Option<TopologySnapshot> {
        let enrichment_facts = self.adapter.query_output_facts();
        let mut candidate_snapshots = Vec::new();
        let mut current_native_ids = Vec::new();

        for proto in protocol_outputs {
            current_native_ids.push(proto.native_id);

            let make = if proto.make.is_empty() {
                None
            } else {
                Some(proto.make.clone())
            };
            let model = if proto.model.is_empty() {
                None
            } else {
                Some(proto.model.clone())
            };

            let mut identity = OutputIdentity {
                make,
                model,
                physical_size_mm: Some(proto.physical_size),
                serial: None,
            };

            let mut preferred = false;

            // Merge unambiguous fresh compositor enrichment facts
            if let Some(name) = &proto.name {
                if let Some(fact) = find_unambiguous_fact(&enrichment_facts, name) {
                    if let Some(serial) = &fact.serial {
                        if !serial.is_empty() {
                            identity.serial = Some(serial.clone());
                        }
                    }
                    if let Some(make) = &fact.make {
                        if !make.is_empty() && identity.make.as_ref().is_none_or(|s| s.is_empty()) {
                            identity.make = Some(make.clone());
                        }
                    }
                    if let Some(model) = &fact.model {
                        if !model.is_empty() && identity.model.as_ref().is_none_or(|s| s.is_empty())
                        {
                            identity.model = Some(model.clone());
                        }
                    }
                    if fact.focused {
                        preferred = true;
                    }
                }
            }

            let output_id = if let Some(&existing_id) = self.id_by_native_id.get(&proto.native_id) {
                existing_id
            } else {
                let new_id = self.registry.allocate_id();
                self.id_by_native_id.insert(proto.native_id, new_id);
                new_id
            };

            candidate_snapshots.push(OutputSnapshot {
                id: output_id,
                name: proto.name.clone(),
                identity,
                logical_position: proto.logical_position,
                logical_size: proto.logical_size,
                logical_role: None,
                scale: proto.scale_factor,
                preferred,
            });
        }

        // Prune disappeared native IDs from routing table before committing
        let removed_native_ids: Vec<u32> = self
            .id_by_native_id
            .keys()
            .copied()
            .filter(|id| !current_native_ids.contains(id))
            .collect();

        for native_id in removed_native_ids {
            if let Some(output_id) = self.id_by_native_id.remove(&native_id) {
                self.native_by_id.remove(&output_id);
            }
        }

        self.registry.commit_topology(candidate_snapshots)
    }

    /// Synchronizes output state from SCTK OutputState and enriches with compositor facts.
    pub fn refresh_sctk(&mut self, output_state: &OutputState) -> Option<TopologySnapshot> {
        let mut protocol_outputs = Vec::new();
        let mut wl_map = HashMap::new();

        for output in output_state.outputs() {
            let Some(info) = output_state.info(&output) else {
                continue;
            };
            wl_map.insert(info.id, output.clone());
            protocol_outputs.push(ProtocolOutput {
                native_id: info.id,
                name: info.name.clone(),
                make: info.make.clone(),
                model: info.model.clone(),
                physical_size: info.physical_size,
                logical_position: info.logical_position,
                logical_size: info.logical_size,
                scale_factor: info.scale_factor,
            });
        }

        let result = self.sync_protocol_facts(&protocol_outputs);

        // Update live WlOutput proxy handles
        self.native_by_id.clear();
        for (native_id, wl_output) in wl_map {
            if let Some(&output_id) = self.id_by_native_id.get(&native_id) {
                self.native_by_id.insert(output_id, wl_output);
            }
        }

        result
    }

    /// Removes a WlOutput synchronously from active registry and native routing maps.
    pub fn remove_wl_output(
        &mut self,
        wl_output: &WlOutput,
    ) -> Option<(OutputId, OutputSnapshot, TopologySnapshot)> {
        let output_id = self.find_id_by_wl_output(wl_output)?;
        self.native_by_id.remove(&output_id);
        self.id_by_native_id.retain(|_, id| *id != output_id);
        let (snapshot, new_topo) = self.registry.remove(output_id)?;
        Some((output_id, snapshot, new_topo))
    }

    /// Removes a native ID synchronously for tests or headless environments.
    pub fn remove_native_id(
        &mut self,
        native_id: u32,
    ) -> Option<(OutputId, OutputSnapshot, TopologySnapshot)> {
        let output_id = self.id_by_native_id.remove(&native_id)?;
        self.native_by_id.remove(&output_id);
        let (snapshot, new_topo) = self.registry.remove(output_id)?;
        Some((output_id, snapshot, new_topo))
    }

    /// Merges enrichment facts into existing outputs without inserting or deleting outputs.
    pub fn merge_enrichment_facts(&mut self, facts: &[CompositorOutputFacts]) -> bool {
        let mut candidate_outputs = self.registry.active_outputs.clone();
        let mut changed = false;

        for snap in &mut candidate_outputs {
            let Some(name) = &snap.name else {
                continue;
            };
            let Some(fact) = find_unambiguous_fact(facts, name) else {
                continue;
            };

            if let Some(serial) = &fact.serial {
                if !serial.is_empty() && snap.identity.serial.as_deref() != Some(serial) {
                    snap.identity.serial = Some(serial.clone());
                    changed = true;
                }
            }
            if let Some(make) = &fact.make {
                if !make.is_empty() && snap.identity.make.as_ref().is_none_or(|s| s.is_empty()) {
                    snap.identity.make = Some(make.clone());
                    changed = true;
                }
            }
            if let Some(model) = &fact.model {
                if !model.is_empty() && snap.identity.model.as_ref().is_none_or(|s| s.is_empty()) {
                    snap.identity.model = Some(model.clone());
                    changed = true;
                }
            }
            if fact.focused != snap.preferred {
                snap.preferred = fact.focused;
                changed = true;
            }
        }

        if changed {
            self.registry.commit_topology(candidate_outputs).is_some()
        } else {
            false
        }
    }

    pub fn add_output_for_test(&mut self, new_output: NewOutput) -> OutputId {
        self.registry.add(new_output)
    }

    pub fn update_output_for_test(&mut self, snapshot: OutputSnapshot) -> bool {
        self.registry.update(snapshot)
    }

    pub fn remove_output_for_test(
        &mut self,
        id: OutputId,
    ) -> Option<(OutputSnapshot, TopologySnapshot)> {
        self.registry.remove(id)
    }
}
