use crate::domain::host::{
    ExclusiveZonePolicy, HostLayer, HostMode, HostSize, KeyboardInteractivityPolicy,
};
use crate::domain::ids::{HostId, OutputId};
use crate::domain::placement::{Anchor, PlacedSurface};
use crate::platform::render_bridge::OwlenWindowAdapter;
use slint::platform::software_renderer::{PremultipliedRgbaColor, TargetPixel};
use slint::platform::{WindowAdapter, WindowEvent};
use slint::PhysicalSize;
use smithay_client_toolkit::compositor::{CompositorState, FrameCallbackData, SurfaceData};
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor as LayerAnchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface,
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
use smithay_client_toolkit::shm::Shm;
use std::collections::{BTreeMap, HashMap};
use wayland_client::protocol::{wl_output, wl_shm, wl_surface};
use wayland_client::QueueHandle;

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

impl HostLayer {
    pub fn as_wlr_layer(self) -> Layer {
        match self {
            HostLayer::Background => Layer::Background,
            HostLayer::Bottom => Layer::Bottom,
            HostLayer::Top => Layer::Top,
            HostLayer::Overlay => Layer::Overlay,
        }
    }
}

impl KeyboardInteractivityPolicy {
    pub fn as_wlr_keyboard_interactivity(self) -> KeyboardInteractivity {
        match self {
            KeyboardInteractivityPolicy::None => KeyboardInteractivity::None,
            KeyboardInteractivityPolicy::OnDemand => KeyboardInteractivity::OnDemand,
            KeyboardInteractivityPolicy::Exclusive => KeyboardInteractivity::Exclusive,
        }
    }
}

pub fn anchor_to_layer_anchor(anchor: Anchor) -> LayerAnchor {
    match anchor {
        Anchor::TopLeft => LayerAnchor::TOP | LayerAnchor::LEFT,
        Anchor::TopRight => LayerAnchor::TOP | LayerAnchor::RIGHT,
        Anchor::BottomLeft => LayerAnchor::BOTTOM | LayerAnchor::LEFT,
        Anchor::BottomRight => LayerAnchor::BOTTOM | LayerAnchor::RIGHT,
        Anchor::Top => LayerAnchor::TOP | LayerAnchor::LEFT | LayerAnchor::RIGHT,
        Anchor::Bottom => LayerAnchor::BOTTOM | LayerAnchor::LEFT | LayerAnchor::RIGHT,
        Anchor::Left => LayerAnchor::LEFT | LayerAnchor::TOP | LayerAnchor::BOTTOM,
        Anchor::Right => LayerAnchor::RIGHT | LayerAnchor::TOP | LayerAnchor::BOTTOM,
        Anchor::Center => LayerAnchor::empty(),
    }
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

/// Authoritative native surface instance owned exclusively by SurfaceManager.
pub struct LiveSurfaceInstance {
    pub token: SurfaceToken,
    pub spec: SurfaceSpec,
    pub surface: wl_surface::WlSurface,
    pub layer: LayerSurface,
    pub output_id: OutputId,
    pub anchor: Anchor,
    pub margins: (i32, i32, i32, i32),
    pub requested_size: (u32, u32),
    pub scale: i32,
    pub dispatched_scale: i32,
    pub configured: Option<(u32, u32)>,
    pub render_size: Option<(u32, u32)>,
    pub frame_pending: bool,
    pub pool: Option<SlotPool>,
    pub buffer: Option<Buffer>,
}

/// Authoritative platform manager owning native Wayland/layer-surface instances,
/// generation progression, and token-based callback routing.
pub struct SurfaceManager {
    registry: SurfaceGenerationRegistry,
    instances: HashMap<HostId, LiveSurfaceInstance>,
    routes_by_surface: HashMap<wl_surface::WlSurface, SurfaceToken>,
}

impl Default for SurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter bundle for creating or reattaching a native surface instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSurfaceParams {
    pub host_id: HostId,
    pub spec: SurfaceSpec,
    pub output_id: OutputId,
    pub output_scale: i32,
    pub anchor: Anchor,
    pub margins: (i32, i32, i32, i32),
    pub requested_size: (u32, u32),
}

impl SurfaceManager {
    pub fn new() -> Self {
        Self {
            registry: SurfaceGenerationRegistry::new(),
            instances: HashMap::new(),
            routes_by_surface: HashMap::new(),
        }
    }

    pub fn generation_registry(&self) -> &SurfaceGenerationRegistry {
        &self.registry
    }

    pub fn generation_registry_mut_for_test(&mut self) -> &mut SurfaceGenerationRegistry {
        &mut self.registry
    }

    pub fn current_token(&self, host_id: &HostId) -> Option<SurfaceToken> {
        self.registry.current_token(host_id)
    }

    pub fn is_current(&self, token: &SurfaceToken) -> bool {
        self.registry.validate(&token.host_id, token)
    }

    pub fn has_native_instance(&self, host_id: &HostId) -> bool {
        self.instances.contains_key(host_id)
    }

    pub fn get_instance(&self, host_id: &HostId) -> Option<&LiveSurfaceInstance> {
        self.instances.get(host_id)
    }

    pub fn get_instance_mut(&mut self, host_id: &HostId) -> Option<&mut LiveSurfaceInstance> {
        self.instances.get_mut(host_id)
    }

    pub fn find_token_by_surface(&self, surface: &wl_surface::WlSurface) -> Option<SurfaceToken> {
        self.routes_by_surface.get(surface).cloned()
    }

    pub fn find_token_by_layer(&self, layer: &LayerSurface) -> Option<SurfaceToken> {
        self.routes_by_surface.get(layer.wl_surface()).cloned()
    }

    pub fn output_id_for_host(&self, host_id: &HostId) -> Option<OutputId> {
        self.instances.get(host_id).map(|inst| inst.output_id)
    }

    /// Retires an active native instance for `host_id` with strict teardown ordering:
    /// 1. Remove native routing entries first
    /// 2. Invalidate/take current SurfaceToken from generation registry
    /// 3. Remove native instance and drop layer before surface
    pub fn retire(&mut self, host_id: &HostId) -> Option<SurfaceToken> {
        if let Some(instance) = self.instances.remove(host_id) {
            self.routes_by_surface.remove(&instance.surface);
            drop(instance);
        }
        self.registry.retire(host_id)
    }

    /// Creates and commits a native surface instance under a strictly monotonic SurfaceToken.
    pub fn attach<State>(
        &mut self,
        params: NativeSurfaceParams,
        wl_output: &wl_output::WlOutput,
        compositor: &CompositorState,
        layer_shell: &LayerShell,
        qh: &QueueHandle<State>,
    ) -> SurfaceToken
    where
        State: wayland_client::Dispatch<wl_surface::WlSurface, SurfaceData<()>>
            + wayland_client::Dispatch<
                wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
                smithay_client_toolkit::shell::wlr_layer::LayerSurfaceData,
            > + 'static,
    {
        self.retire(&params.host_id);

        let token = self.registry.allocate(params.host_id.clone());

        let wl_surface = compositor.create_surface(qh);
        let layer_anchor = match params.spec.mode {
            HostMode::Cover => LayerAnchor::TOP | LayerAnchor::LEFT | LayerAnchor::RIGHT,
            _ => anchor_to_layer_anchor(params.anchor),
        };
        let layer_surface = layer_shell.create_layer_surface(
            qh,
            wl_surface.clone(),
            params.spec.layer.as_wlr_layer(),
            Some(&params.spec.namespace),
            Some(wl_output),
        );

        layer_surface.set_anchor(layer_anchor);
        layer_surface.set_size(params.requested_size.0, params.requested_size.1);
        let ez = match params.spec.exclusive_zone {
            ExclusiveZonePolicy::None => 0,
            ExclusiveZonePolicy::Fixed(px) => px as i32,
            ExclusiveZonePolicy::Auto => params.requested_size.1 as i32,
        };
        layer_surface.set_exclusive_zone(ez);
        layer_surface.set_margin(
            params.margins.0,
            params.margins.1,
            params.margins.2,
            params.margins.3,
        );
        layer_surface.set_keyboard_interactivity(
            params
                .spec
                .keyboard_interactivity
                .as_wlr_keyboard_interactivity(),
        );
        layer_surface.commit();

        let scale = params.output_scale.max(1);
        wl_surface.set_buffer_scale(scale);

        self.routes_by_surface
            .insert(wl_surface.clone(), token.clone());

        let instance = LiveSurfaceInstance {
            token: token.clone(),
            spec: params.spec,
            surface: wl_surface,
            layer: layer_surface,
            output_id: params.output_id,
            anchor: params.anchor,
            margins: params.margins,
            requested_size: params.requested_size,
            scale,
            dispatched_scale: 0,
            configured: None,
            render_size: None,
            frame_pending: false,
            pool: None,
            buffer: None,
        };

        self.instances.insert(params.host_id, instance);
        token
    }

    /// Reconfigures layer anchor and margins without recreating the surface instance.
    pub fn reconfigure_placement(
        &mut self,
        host_id: &HostId,
        anchor: Anchor,
        margins: (i32, i32, i32, i32),
    ) -> bool {
        let Some(inst) = self.instances.get_mut(host_id) else {
            return false;
        };
        inst.anchor = anchor;
        inst.margins = margins;
        inst.layer.set_anchor(anchor_to_layer_anchor(anchor));
        inst.layer
            .set_margin(margins.0, margins.1, margins.2, margins.3);
        inst.layer.commit();
        true
    }

    /// Handles `configure` event from compositor, validating token generation.
    pub fn handle_configure(
        &mut self,
        layer: &LayerSurface,
        new_size: (u32, u32),
    ) -> Option<SurfaceToken> {
        let token = self.find_token_by_layer(layer)?;
        if !self.registry.validate(&token.host_id, &token) {
            return None;
        }
        let inst = self.instances.get_mut(&token.host_id)?;
        if inst.token != token {
            return None;
        }
        inst.configured = Some(new_size);
        Some(token)
    }

    /// Handles `frame` callback from compositor, validating token generation.
    pub fn handle_frame(&mut self, surface: &wl_surface::WlSurface) -> Option<SurfaceToken> {
        let token = self.routes_by_surface.get(surface)?.clone();
        if !self.registry.validate(&token.host_id, &token) {
            return None;
        }
        let inst = self.instances.get_mut(&token.host_id)?;
        if inst.token != token {
            return None;
        }
        inst.frame_pending = false;
        Some(token)
    }

    /// Handles `scale_factor_changed` from compositor, validating token generation.
    pub fn handle_scale(
        &mut self,
        surface: &wl_surface::WlSurface,
        new_scale: i32,
    ) -> Option<(SurfaceToken, i32)> {
        let token = self.routes_by_surface.get(surface)?.clone();
        if !self.registry.validate(&token.host_id, &token) {
            return None;
        }
        let inst = self.instances.get_mut(&token.host_id)?;
        if inst.token != token {
            return None;
        }
        let scale = new_scale.max(1);
        inst.scale = scale;
        inst.surface.set_buffer_scale(scale);
        Some((token, scale))
    }

    /// Synchronizes native Slint window size and dynamic preferred size.
    pub fn sync_size(&mut self, host_id: &HostId, adapter: &OwlenWindowAdapter) {
        let Some(inst) = self.instances.get_mut(host_id) else {
            return;
        };
        let preferred = adapter.preferred.get();
        let scale_i32 = inst.scale.max(1);
        let scale = scale_i32 as f32;

        if inst.dispatched_scale != scale_i32 {
            let _ = adapter
                .window
                .try_dispatch_event(WindowEvent::ScaleFactorChanged {
                    scale_factor: scale,
                });
            inst.dispatched_scale = scale_i32;
        }

        let target_size = match inst.spec.size {
            HostSize::OutputWidth { height } => {
                let (cw, ch) = inst.configured.unwrap_or((0, height));
                let w = if cw == 0 { 1 } else { cw };
                let h = if ch == 0 { height } else { ch };
                ((w as f32 * scale) as u32, (h as f32 * scale) as u32)
            }
            _ => {
                let lw = preferred.width.max(1.0) as u32;
                let lh = preferred.height.max(1.0) as u32;
                let (cw, ch) = inst.configured.unwrap_or((lw, lh));
                let w = if cw == 0 { lw } else { cw };
                let h = if ch == 0 { lh } else { ch };
                if inst.requested_size != (lw, lh) {
                    inst.layer.set_size(lw, lh);
                    inst.layer.commit();
                    inst.requested_size = (lw, lh);
                }
                ((w as f32 * scale) as u32, (h as f32 * scale) as u32)
            }
        };

        if inst.render_size != Some(target_size) {
            adapter
                .window
                .set_size(PhysicalSize::new(target_size.0, target_size.1));
            inst.render_size = Some(target_size);
        }
    }

    /// Renders software buffer and attaches it to the native Wayland surface.
    pub fn draw_surface<State>(
        &mut self,
        host_id: &HostId,
        adapter: &OwlenWindowAdapter,
        shm: &Shm,
        qh: &QueueHandle<State>,
    ) where
        State: wayland_client::Dispatch<
                wayland_client::protocol::wl_callback::WlCallback,
                FrameCallbackData,
            > + 'static,
    {
        let Some(inst) = self.instances.get_mut(host_id) else {
            return;
        };
        if inst.configured.is_none() || inst.frame_pending {
            return;
        }
        let scale = inst.scale.max(1);
        let preferred = adapter.preferred.get();
        let (cw, ch) = inst.configured.unwrap();
        let (lw, lh) = match inst.spec.size {
            HostSize::OutputWidth { height } => (
                if cw == 0 { 1 } else { cw },
                if ch == 0 { height } else { ch },
            ),
            _ => {
                let pw = preferred.width.max(1.0) as u32;
                let ph = preferred.height.max(1.0) as u32;
                (if cw == 0 { pw } else { cw }, if ch == 0 { ph } else { ch })
            }
        };
        let w = (lw as i32) * scale;
        let h = (lh as i32) * scale;
        if w <= 0 || h <= 0 {
            return;
        }

        let mut rendered = None;
        let redrawn = adapter.window.draw_if_needed(|renderer| {
            let mut tmp = vec![PremultipliedRgbaColor::background(); (w * h) as usize];
            renderer.render(&mut tmp, w as usize);
            rendered = Some(tmp);
        });
        if !redrawn {
            return;
        }
        let Some(tmp) = rendered else {
            return;
        };

        let bytes = (w * h * 4) as usize;
        if inst.pool.is_none() {
            inst.pool = SlotPool::new(bytes.max(4096), shm).ok();
        }
        let Some(pool) = inst.pool.as_mut() else {
            WindowAdapter::request_redraw(&*adapter.window);
            return;
        };
        let (buffer, canvas) = match pool.create_buffer(w, h, w * 4, wl_shm::Format::Argb8888) {
            Ok(pair) => pair,
            Err(_) => {
                let _ = pool.resize(bytes * 2);
                let Ok(pair) = pool.create_buffer(w, h, w * 4, wl_shm::Format::Argb8888) else {
                    WindowAdapter::request_redraw(&*adapter.window);
                    return;
                };
                pair
            }
        };

        for (dst, src) in canvas.as_chunks_mut::<4>().0.iter_mut().zip(tmp.iter()) {
            dst[0] = src.blue;
            dst[1] = src.green;
            dst[2] = src.red;
            dst[3] = src.alpha;
        }

        if buffer.attach_to(&inst.surface).is_err() {
            WindowAdapter::request_redraw(&*adapter.window);
            return;
        }
        inst.surface.damage_buffer(0, 0, w, h);
        inst.surface
            .frame(qh, FrameCallbackData(inst.surface.clone()));
        inst.layer.commit();
        inst.buffer = Some(buffer);
        inst.frame_pending = true;
    }
}

impl From<&crate::domain::host::HostSpec> for SurfaceSpec {
    fn from(spec: &crate::domain::host::HostSpec) -> Self {
        Self {
            namespace: spec.namespace.clone(),
            layer: spec.layer,
            mode: spec.mode.clone(),
            size: spec.size,
            exclusive_zone: spec.exclusive_zone,
            keyboard_interactivity: spec.keyboard_interactivity,
        }
    }
}
