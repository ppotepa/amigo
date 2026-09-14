use crate::state::{ObjectSettings, Settings, style_preset_id};
mod layers;
mod model;
use amigo_render_api::{MeshDrawCommand, NprBackgroundCommand, NprDrawCommand};
use amigo_render_npr::*;
use glam::{Mat4, Quat, Vec3};
use model::ModelGeometry;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};
pub const NPR_PLAYGROUND_EXTRACTOR_ID: &str = "amigo.gfx.npr-playground.extractor";

#[derive(Clone)]
struct SourceCommand {
    object_id: String,
    temporal_scope: u64,
    command: NprDrawCommand,
}

#[derive(Clone, PartialEq)]
struct ObjectPacketKey {
    surface: NprSurfaceContentId,
    object: ObjectSettings,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    layers: LayerGeometryKey,
    seed: u64,
    epoch: u32,
    strength: f32,
    debug: NprDebugView,
    highlighted: bool,
}

/// Properties that alter extracted packet geometry are kept separate from
/// compositor-only layer state. Changing opacity, colour, blend, masks,
/// targets or group presentation can therefore reuse the expensive packet.
#[derive(Clone, PartialEq)]
struct LayerGeometryKey(
    Vec<(
        String,
        NprGeometrySource,
        Option<StrokeTool>,
        Option<NprPaintMedium>,
        Option<BrushInstance>,
        bool,
        Option<NprHatchSettings>,
        Option<NprLineSettings>,
    )>,
);

fn layer_geometry_key(layers: &NprStyleLayers) -> LayerGeometryKey {
    LayerGeometryKey(
        layers
            .layers
            .iter()
            .map(|layer| {
                (
                    layer.id.clone(),
                    layer.source,
                    layer.tool,
                    layer.paint,
                    layer.brush.clone(),
                    layer.enabled,
                    layer.hatch,
                    layer.line,
                )
            })
            .collect(),
    )
}

/// A drawing frame can contain several independent NPR packets. Individual packets
/// already have a CPU payload ceiling, but without a scene-level cap their
/// combined upload could still make the backend silently skip late strokes.
/// This budget deliberately leaves headroom below the WGPU upload ceiling.
const MAX_DRAWING_STROKE_DATA_BYTES: usize = 48 * 1024 * 1024;

/// A stable source-surface result selected from the current NPR scene.
///
/// This is intentionally a plugin-domain type: it names the authored object,
/// while `NprSurfaceRayHit` remains reusable by every consumer of
/// `amigo-render-npr`.
#[derive(Debug, Clone, PartialEq)]
pub struct NprSurfacePick {
    pub object_id: String,
    pub anchor: crate::state::ConstructionAnchorSettings,
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct NprPlaygroundRenderService {
    geometry: Arc<Mutex<BTreeMap<String, ModelGeometry>>>,
    source: Mutex<Vec<SourceCommand>>,
    packet_keys: Mutex<BTreeMap<String, ObjectPacketKey>>,
    packet_builds: std::sync::atomic::AtomicU64,
    output: Mutex<(Vec<NprDrawCommand>, Option<NprBackgroundCommand>)>,
    scene_is_npr: std::sync::atomic::AtomicBool,
    last_input: Mutex<Option<(Settings, [u32; 2])>>,
    temporal: Mutex<DrawingHistory>,
    variants: Mutex<StrokeVariantClock>,
    lod: Mutex<BTreeMap<String, HatchLodState>>,
    layer_diagnostics: Mutex<BTreeMap<String, NprLayerDiagnostics>>,
}
impl Default for NprPlaygroundRenderService {
    fn default() -> Self {
        Self {
            geometry: Arc::new(Mutex::new(
                [
                    ("cube", NprGeometry::canonical_cube()),
                    ("wedge", NprGeometry::wedge()),
                    ("cylinder", NprGeometry::cylinder(24)),
                    ("sphere", NprGeometry::icosphere()),
                ]
                .into_iter()
                .map(|(name, g)| (name.into(), ModelGeometry::builtin(g)))
                .collect(),
            )),
            source: Mutex::new(vec![]),
            packet_keys: Mutex::default(),
            packet_builds: std::sync::atomic::AtomicU64::new(0),
            output: Mutex::new((vec![], None)),
            scene_is_npr: std::sync::atomic::AtomicBool::new(false),
            last_input: Mutex::new(None),
            temporal: Mutex::new(DrawingHistory::default()),
            variants: Mutex::new(StrokeVariantClock::default()),
            lod: Mutex::new(BTreeMap::new()),
            layer_diagnostics: Mutex::default(),
        }
    }
}
impl NprPlaygroundRenderService {
    /// Returns whether the active scene owns the world presentation through
    /// the NPR extractor. The regular mesh bridge uses this to avoid leaking
    /// a standard shaded frame while the NPR packet is still preparing.
    pub fn scene_is_npr(&self) -> bool {
        self.scene_is_npr.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn set_scene_is_npr(&self, active: bool) {
        self.scene_is_npr
            .store(active, std::sync::atomic::Ordering::Release);
    }

    /// Immutable surfaces and their revision caches are shared; every camera
    /// retains independent packet, LOD, variant and fade state.
    pub fn fork_view(&self) -> Self {
        Self {
            geometry: self.geometry.clone(),
            ..Self::default()
        }
    }
    pub fn load_model(&self, id: &str, path: &Path) -> Result<(), String> {
        if self.geometry.lock().unwrap().contains_key(id) {
            return Ok(());
        }
        let mesh = amigo_3d_mesh::load_gltf_geometry(path)?;
        self.geometry
            .lock()
            .unwrap()
            .insert(id.into(), ModelGeometry::imported(mesh)?);
        Ok(())
    }
    pub fn animations(&self, model: &str) -> Vec<amigo_3d_mesh::MeshAnimationClip> {
        self.geometry
            .lock()
            .unwrap()
            .get(model)
            .map(ModelGeometry::animations)
            .unwrap_or_default()
    }
    pub fn clear(&self) {
        *self.last_input.lock().unwrap() = None;
        self.source.lock().unwrap().clear();
        self.packet_keys.lock().unwrap().clear();
        self.packet_builds
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.temporal.lock().unwrap().clear();
        self.variants.lock().unwrap().clear();
        self.lod.lock().unwrap().clear();
        self.layer_diagnostics.lock().unwrap().clear();
        *self.output.lock().unwrap() = (vec![], None);
    }
    pub fn load_models(&self, root: &Path) -> Result<(), String> {
        for (name, path) in [
            ("suzanne", "assets/models/suzanne/Suzanne.gltf"),
            ("avocado", "assets/models/avocado/Avocado.glb"),
        ] {
            self.load_model(name, &root.join(path))?;
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Option<NprDrawCommand> {
        self.output.lock().unwrap().0.first().cloned()
    }
    pub fn commands(&self) -> Vec<NprDrawCommand> {
        self.output.lock().unwrap().0.clone()
    }

    /// Publishes a fully prepared scene snapshot. Runtime preparation owns
    /// packet construction; render extraction only reads this completed output.
    pub fn publish_scene_snapshot(
        &self,
        commands: Vec<NprDrawCommand>,
        background: Option<NprBackgroundCommand>,
    ) {
        *self.output.lock().unwrap() = (commands, background);
    }
    pub fn layer_diagnostics(&self) -> BTreeMap<String, NprLayerDiagnostics> {
        self.layer_diagnostics.lock().unwrap().clone()
    }
    /// Borrow the prepared view for synchronous backend submission without
    /// cloning its stroke/vertex packets. The callback must not reenter us.
    pub fn with_commands<T>(
        &self,
        read: impl FnOnce(&[NprDrawCommand], Option<NprBackgroundCommand>) -> T,
    ) -> T {
        let output = self.output.lock().unwrap();
        read(&output.0, output.1)
    }
    pub fn stats(&self) -> BTreeMap<String, u64> {
        let output = self.output.lock().unwrap();
        let mut stats = BTreeMap::new();
        stats.insert(
            "packet_builds".into(),
            self.packet_builds
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        for command in &output.0 {
            let s = &command.packet.stats;
            for (key, value) in [
                ("geometry", s.geometry),
                ("surface_source_vertices", s.surface_source_vertices),
                ("surface_proxy_vertices", s.surface_proxy_vertices),
                ("surface_source_triangles", s.surface_source_triangles),
                ("surface_proxy_triangles", s.surface_proxy_triangles),
                ("topology_edges", s.topology_edges),
                ("feature_segments", s.feature_segments),
                ("feature_candidates", s.feature_candidates),
                ("feature_rejected", s.feature_rejected),
                ("smooth_contour_rejected", s.smooth_contour_rejected),
                ("suggestive_contour_rejected", s.suggestive_contour_rejected),
                ("smooth_contour_spans", s.smooth_contour_spans),
                ("suggestive_contour_spans", s.suggestive_contour_spans),
                ("silhouettes", s.silhouettes),
                ("creases", s.creases),
                ("strokes", s.strokes),
                ("underpainting_triangles", s.underpainting_triangles),
                ("stroke_vertices", s.stroke_vertices),
                ("stroke_indices", s.stroke_indices),
                ("hatching_strokes", s.hatching_strokes),
                ("form_line_strokes", s.form_line_strokes),
                ("hatching_correction_strokes", s.hatching_correction_strokes),
                (
                    "graphite_mass_milli",
                    (s.graphite_mass.max(0.0) * 1000.0).round() as usize,
                ),
                ("hatching_candidates", s.hatching_candidates),
                ("hatching_rejected", s.hatching_rejected),
                (
                    "hatching_confidence_rejected",
                    s.hatching_confidence_rejected,
                ),
                ("construction_marks", s.construction_marks),
                ("construction_rejected", s.construction_rejected),
                ("stroke_budget_rejected", s.stroke_budget_rejected),
                ("temporal_retained_strokes", s.temporal_retained_strokes),
                ("temporal_entering_strokes", s.temporal_entering_strokes),
                ("stroke_data_bytes", s.stroke_data_bytes),
            ] {
                *stats.entry(key.into()).or_insert(0) += value as u64;
            }
            stats.insert("viewport_width".into(), s.viewport[0] as u64);
            stats.insert("viewport_height".into(), s.viewport[1] as u64);
            if s.hatching_budget_exhausted {
                *stats.entry("hatching_budget_exhausted".into()).or_insert(0) += 1;
            }
            if s.stroke_budget_exhausted {
                *stats.entry("stroke_budget_exhausted".into()).or_insert(0) += 1;
            }
        }
        stats.insert(
            "hatching_lod_tier".into(),
            output
                .0
                .iter()
                .map(|command| u64::from(command.packet.stats.hatching_lod_tier))
                .max()
                .unwrap_or(0),
        );
        stats.insert(
            "gesture_variant_epoch".into(),
            output
                .0
                .iter()
                .map(|command| u64::from(command.packet.stats.gesture_variant_epoch))
                .max()
                .unwrap_or(0),
        );
        stats
    }
    pub fn background(&self) -> Option<NprBackgroundCommand> {
        self.output.lock().unwrap().1
    }

    /// Picks the nearest currently visible authored object at a viewport pixel.
    ///
    /// The source mesh, rather than the selected smooth proxy, is queried so
    /// the returned anchor can be placed directly into scene authoring data.
    pub fn pick_surface(
        &self,
        settings: &Settings,
        viewport: [u32; 2],
        screen: glam::Vec2,
    ) -> Option<NprSurfacePick> {
        let camera = world_camera(settings, viewport)?;
        let (origin, direction) = camera.ray_from_screen(screen, viewport_vec(viewport))?;
        let mut cache = self.geometry.lock().unwrap();
        settings
            .objects
            .iter()
            .filter(|(id, object)| object.visible && **id == settings.selected)
            .filter_map(|(id, object)| {
                let transform = object_transform(object);
                let inverse = transform.inverse();
                let hit = cache
                    .get_mut(&object.model)?
                    .prepared(settings.playback.as_ref())
                    .ok()?
                    .source()
                    .raycast(
                        inverse.transform_point3(origin),
                        inverse.transform_vector3(direction).normalize_or_zero(),
                    )?;
                let position = transform.transform_point3(hit.position);
                let distance = (position - origin).length();
                distance.is_finite().then_some((
                    distance,
                    NprSurfacePick {
                        object_id: id.clone(),
                        anchor: crate::state::ConstructionAnchorSettings {
                            triangle: hit.anchor.triangle,
                            barycentric: hit.anchor.barycentric,
                        },
                        position,
                        normal: transform.transform_vector3(hit.normal).normalize_or_zero(),
                    },
                ))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, pick)| pick)
    }
    /// Rebuilds a frozen reference frame. Tests and deterministic screenshots
    /// use this entry point; interactive extraction uses `rebuild_with_delta`.
    pub fn rebuild(&self, settings: &Settings, viewport: [u32; 2]) -> Result<(), String> {
        self.rebuild_internal(settings, viewport, 0.0, false, true)
    }

    /// Applies session-owned temporal continuity after rebuilding (only when
    /// settings/viewport changed) or reusing a pure reference packet.
    pub fn rebuild_with_delta(
        &self,
        settings: &Settings,
        viewport: [u32; 2],
        delta_seconds: f32,
    ) -> Result<(), String> {
        self.rebuild_internal(settings, viewport, delta_seconds, true, true)
    }

    /// Builds NPR packets from the hydrated scene geometry. Scene meshes remain
    /// the source of truth; this plugin only turns their explicit NPR profile
    /// into neutral render commands.
    pub fn rebuild_scene_with_delta(
        &self,
        settings: &Settings,
        meshes: &[MeshDrawCommand],
        viewport: [u32; 2],
        delta_seconds: f32,
    ) -> Result<(), String> {
        let mut scene = settings.clone();
        scene.objects.clear();
        let mut geometry_cache = self.geometry.lock().unwrap();
        for command in meshes {
            let Some(geometry) = command.mesh.geometry.as_ref() else {
                continue;
            };
            let id = command.entity_name.clone();
            // City scenes commonly instance one GLB dozens of times. Keep one
            // prepared NPR surface per asset, never one per entity or frame.
            let model = command.mesh.mesh_asset.as_str().to_owned();
            if !geometry_cache.contains_key(&model) {
                geometry_cache.insert(
                    model.clone(),
                    ModelGeometry::builtin(NprGeometry::from_indexed(
                        &geometry.positions,
                        &geometry.indices,
                    )?),
                );
            }
            let transform = command.mesh.transform;
            scene.objects.insert(
                id.clone(),
                ObjectSettings {
                    model,
                    material_base_color: glam::Vec4::ONE,
                    surface_intent: Default::default(),
                    surface_mode: Default::default(),
                    surface_subdivision_level: 0,
                    smooth_weld_relative_tolerance: 0.001,
                    visible: true,
                    rotating: false,
                    position: glam::Vec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    rotation: glam::Vec3::new(
                        transform.rotation_euler.x.to_degrees(),
                        transform.rotation_euler.y.to_degrees(),
                        transform.rotation_euler.z.to_degrees(),
                    ),
                    scale: transform.scale.x,
                    angular_speed: glam::Vec3::ZERO,
                    gesture_variant: 0,
                    style_overrides: Default::default(),
                    style_layer_overrides: Default::default(),
                    construction_marks: vec![],
                },
            );
        }
        drop(geometry_cache);
        scene.selected = scene.objects.keys().next().cloned().unwrap_or_default();
        // The scene adapter builds one transient entry per Mesh3D command.
        // That is valid render input, but not a Drawing Studio document.
        self.rebuild_internal(&scene, viewport, delta_seconds, true, false)
    }

    fn rebuild_internal(
        &self,
        settings: &Settings,
        viewport: [u32; 2],
        delta_seconds: f32,
        apply_temporal: bool,
        validate_drawing_document: bool,
    ) -> Result<(), String> {
        if viewport.contains(&0) {
            self.clear();
            return Ok(());
        }
        let input_changed = !self
            .last_input
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|(last, size)| last == settings && *size == viewport);
        if input_changed && validate_drawing_document {
            settings.validate()?;
        }
        // Packet construction is deliberately outside the per-frame path. A
        // scene may contain dozens of instances and `build_drawing` is CPU
        // intensive; running it here when neither scene input nor viewport
        // changed starves the window event loop. Temporal presentation below
        // still advances every frame from the last complete packet.
        // The drawing studio retains its continuous redraw semantics. Hydrated
        // runtime scenes (`validate_drawing_document == false`) publish a
        // stable snapshot and rebuild only after an actual scene/camera change.
        if input_changed || validate_drawing_document {
            let yaw = settings.camera_yaw.to_radians();
            let pitch = settings.camera_pitch.to_radians();
            let position = settings.camera_target
                + Vec3::new(
                    yaw.sin() * pitch.cos(),
                    pitch.sin(),
                    yaw.cos() * pitch.cos(),
                ) * settings.camera_distance;
            let camera = PerspectiveCamera {
                position,
                forward: (settings.camera_target - position).normalize(),
                up: Vec3::Y,
                vertical_fov: settings.camera_fov.to_radians(),
                near: 0.05,
                aspect: viewport[0] as f32 / viewport[1] as f32,
            };
            let world_camera = camera;
            let debug = match settings.debug.as_str() {
                "FeatureClasses" => NprDebugView::FeatureClasses,
                "StrokeIds" => NprDebugView::StrokeIds,
                _ => NprDebugView::Final,
            };
            let mut lod = self.lod.lock().unwrap();
            let mut variants = self.variants.lock().unwrap();
            let mut packet_keys = self.packet_keys.lock().unwrap();
            let mut old_source: BTreeMap<_, _> = std::mem::take(&mut *self.source.lock().unwrap())
                .into_iter()
                .map(|entry| (entry.object_id.clone(), entry))
                .collect();
            let mut source = Vec::new();
            for (id, object) in &settings.objects {
                if !object.visible {
                    continue;
                }
                let mut style = object.effective_style(settings.global);
                style.surface_mode = object.surface_intent.resolve_mode(object.surface_mode);
                if object.surface_intent.suppresses_topology_creases() {
                    // Organic source meshes routinely contain triangulation
                    // seams which are not authorial marks.  The intent is
                    // resolved here, before the neutral packet is built.
                    style.smooth_draw_creases = false;
                }
                let mut cache = self.geometry.lock().unwrap();
                let prepared_variants = cache
                    .get_mut(&object.model)
                    .ok_or_else(|| format!("model {} is not prepared", object.model))?
                    .prepared(settings.playback.as_ref())?;
                let source_surface = prepared_variants.source_shared();
                let source_geometry = source_surface.geometry();
                let source_vertices = source_geometry.vertices.len();
                let source_triangles = source_geometry.triangles.len();
                let rotation = object.rotation.map(f32::to_radians);
                let transform = Mat4::from_scale_rotation_translation(
                    Vec3::splat(object.scale),
                    Quat::from_euler(glam::EulerRot::YXZ, rotation.y, rotation.x, rotation.z),
                    object.position,
                );
                let proxy_policy = NprSmoothProxyPolicy {
                    levels: object
                        .surface_intent
                        .resolve_subdivision_level(object.surface_subdivision_level),
                    crease_angle: style.smooth_crease_angle,
                    weld_relative_tolerance: object.smooth_weld_relative_tolerance,
                    ..NprSmoothProxyPolicy::default()
                };
                let prepared = if style.surface_mode == NprSurfaceMode::Smooth {
                    prepared_variants.smooth_proxy(proxy_policy)
                } else {
                    Ok(source_surface.clone())
                }
                .map_err(|error| format!("model {} smooth proxy: {error}", object.model))?;
                drop(cache);
                let world_geometry = prepared.geometry().transformed(transform);
                // NPR source identities live in model space. Transforming the
                // camera and directional light into that space produces the
                // same projected image as transforming every source path into
                // world space, while preserving stable surface coordinates for
                // rotations, translations and uniform scale.
                let inverse = transform.inverse();
                let camera = PerspectiveCamera {
                    position: inverse.transform_point3(world_camera.position),
                    forward: inverse
                        .transform_vector3(world_camera.forward)
                        .normalize_or_zero(),
                    up: inverse
                        .transform_vector3(world_camera.up)
                        .normalize_or_zero(),
                    vertical_fov: world_camera.vertical_fov,
                    near: world_camera.near,
                    aspect: world_camera.aspect,
                };
                // Scope hashes authored intent before its view/local-space
                // adaptation. In particular, object rotation changes the
                // local light vector but must not erase drawing history.
                let temporal_scope = temporal_scope(id, &object.model, style, settings.seed);
                style.light_direction = inverse
                    .transform_vector3(settings.global.light_direction)
                    .normalize_or_zero();
                let preset = style_preset_id(style);
                let motion = if settings.sketch_paused {
                    NprMotionPolicy {
                        mode: StrokeMotionMode::Stable,
                        ..settings.motion
                    }
                } else {
                    settings.motion
                };
                let variant_epoch = variants.advance(
                    temporal_scope,
                    &projected_motion_anchors(&world_geometry, world_camera, viewport),
                    if apply_temporal { delta_seconds } else { 0.0 },
                    motion,
                );
                let variant_epoch = object.gesture_variant.wrapping_add(variant_epoch);
                let variant_strength = if object.gesture_variant == 0 {
                    motion.redraw_strength
                } else {
                    1.0
                };
                let decision = lod.entry(id.clone()).or_default().advance(
                    projected_extent(&world_geometry, world_camera, viewport),
                    HatchLodPolicy::default(),
                );
                style.hatching_spacing *= decision.spacing_multiplier;
                let mut layers = object.effective_layers(&settings.style_layers).clone();
                for layer in &mut layers.layers {
                    if layer.paint.is_none()
                        && matches!(
                            layer.source,
                            NprGeometrySource::Wash | NprGeometrySource::FlatFill
                        )
                    {
                        if let Some(instance) = &layer.brush {
                            layer.paint = settings.brushes.resolve(&instance.brush)?.paint;
                        }
                    }
                }
                let highlighted = false;
                let key = ObjectPacketKey {
                    surface: prepared.content_id(),
                    object: object.clone(),
                    camera,
                    viewport,
                    style,
                    layers: layer_geometry_key(&layers),
                    seed: settings.seed,
                    epoch: variant_epoch,
                    strength: variant_strength,
                    debug,
                    highlighted,
                };
                if packet_keys.get(id) == Some(&key) {
                    if let Some(mut entry) = old_source.remove(id) {
                        // Refresh compositor semantics while retaining the
                        // extracted packet and its tessellated paths.
                        entry.command.layers = layers;
                        let mut diagnostics = self.layer_diagnostics.lock().unwrap();
                        let refreshed = entry.command.layers.diagnostics(
                            &entry.command.packet,
                            &entry.command.packet,
                            0,
                            0,
                        );
                        for (layer_id, mut current) in refreshed {
                            if let Some(previous) = diagnostics.get(&layer_id) {
                                current.source_geometry = previous.source_geometry;
                            }
                            if entry
                                .command
                                .layers
                                .layer(&layer_id)
                                .is_some_and(|layer| !layer.target.includes(id, layer.source))
                            {
                                current.no_effect_reason =
                                    Some(NprLayerNoEffectReason::TargetExcludesGeometry);
                                current.mask_coverage = 0.0;
                            }
                            diagnostics.insert(layer_id, current);
                        }
                        source.push(entry);
                        continue;
                    }
                }
                let construction_marks = object
                    .construction_marks
                    .iter()
                    .map(|mark| mark.resolve(&source_surface))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| format!("object {id} construction marks: {error}"))?;
                let (mut packet, mut diagnostics) = layers::build_drawing(
                    &prepared,
                    &source_surface,
                    camera,
                    viewport,
                    style,
                    variant_seed(
                        object_seed(id, &object.model, settings.seed),
                        variant_epoch,
                        variant_strength,
                    ),
                    debug,
                    &layers,
                    &settings.brushes,
                    &construction_marks,
                )?;
                packet.stats.surface_source_vertices = source_vertices;
                packet.stats.surface_proxy_vertices = prepared.geometry().vertices.len();
                packet.stats.surface_source_triangles = source_triangles;
                packet.stats.surface_proxy_triangles = prepared.geometry().triangles.len();
                packet.stats.hatching_lod_tier = decision.tier;
                packet.stats.gesture_variant_epoch = variant_epoch;
                if highlighted {
                    packet.mark_selection(
                        prepared.geometry(),
                        camera,
                        glam::Vec4::new(0.15, 0.65, 0.85, 1.0),
                    );
                }
                for layer in &layers.layers {
                    if !layer.target.includes(id, layer.source) {
                        if let Some(diagnostic) = diagnostics.get_mut(&layer.id) {
                            diagnostic.no_effect_reason =
                                Some(NprLayerNoEffectReason::TargetExcludesGeometry);
                            diagnostic.mask_coverage = 0.0;
                        }
                    }
                }
                *self.layer_diagnostics.lock().unwrap() = diagnostics;
                source.push(SourceCommand {
                    object_id: id.clone(),
                    temporal_scope,
                    command: NprDrawCommand::with_preset_and_layers(packet, preset, layers)
                        .with_object_id(id.clone())
                        .with_material_base_color(object.material_base_color.to_array()),
                });
                packet_keys.insert(id.clone(), key);
                self.packet_builds
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            packet_keys.retain(|id, _| source.iter().any(|entry| &entry.object_id == id));
            *self.source.lock().unwrap() = source;
            *self.last_input.lock().unwrap() = Some((settings.clone(), viewport));
        }
        let source = self.source.lock().unwrap().clone();
        let commands = if apply_temporal {
            let mut history = self.temporal.lock().unwrap();
            let policy = TemporalPolicy {
                appear_seconds: settings.motion.appearance_fade_seconds,
                ..TemporalPolicy::default()
            };
            history.begin_frame();
            let commands = source
                .into_iter()
                .map(|mut entry| {
                    history.advance_packet_in_frame(
                        entry.temporal_scope,
                        &mut entry.command.packet,
                        delta_seconds,
                        policy,
                    );
                    entry
                })
                .collect();
            history.finish_frame(delta_seconds, policy);
            commands
        } else {
            source
        };
        let paper_color = settings
            .style_layers
            .layers
            .iter()
            .find(|layer| layer.source == NprGeometrySource::Paper)
            .and_then(|layer| match layer.color_source {
                NprLayerColorSource::Constant(color) => Some(color),
                _ => None,
            })
            .unwrap_or(settings.global.paper);
        let commands = retain_strokes_under_budget(commands, &settings.selected)
            .into_iter()
            .map(|mut entry| {
                entry.command.packet.background = paper_color;
                entry.command
            })
            .collect();
        *self.output.lock().unwrap() = (
            commands,
            Some(NprBackgroundCommand {
                color: paper_color.to_array(),
                grain: settings.global.paper_grain,
                tooth: settings.global.paper_tooth,
                seed: settings.seed,
            }),
        );
        Ok(())
    }
    pub fn rebuild_cube(&self, viewport: [u32; 2], seed: u64) {
        let mut settings = Settings::for_scene();
        settings.seed = seed;
        self.rebuild(&settings, viewport)
            .expect("built-in cube is valid");
    }
}

fn retain_strokes_under_budget(commands: Vec<SourceCommand>, selected: &str) -> Vec<SourceCommand> {
    retain_strokes_with_budget(commands, selected, MAX_DRAWING_STROKE_DATA_BYTES)
}

fn retain_strokes_with_budget(
    mut commands: Vec<SourceCommand>,
    selected: &str,
    byte_budget: usize,
) -> Vec<SourceCommand> {
    // The selected model is the active inspection target. Remaining ties are
    // object-id ordered, which keeps the resulting packet deterministic.
    commands.sort_by(|left, right| {
        (right.object_id == selected)
            .cmp(&(left.object_id == selected))
            .then_with(|| left.object_id.cmp(&right.object_id))
    });
    let mut remaining = byte_budget;
    for entry in &mut commands {
        let packet = &mut entry.command.packet;
        let bytes: usize = packet.strokes.iter().map(stroke_data_bytes).sum();
        if bytes <= remaining {
            remaining -= bytes;
            continue;
        }
        let mut retained = Vec::with_capacity(packet.strokes.len());
        let mut rejected = 0usize;
        let mut rejected_tone = 0usize;
        let mut rejected_form_lines = 0usize;
        // `build_packet_for_surface` emits feature strokes before tonal ones.
        // Preserve that authored order, while construction marks and features
        // are always considered ahead of form lines and hatch density within
        // each object.
        let mut lower_priority_allowed = true;
        for role in [
            StrokeRole::Construction,
            StrokeRole::Feature,
            StrokeRole::FormLine,
            StrokeRole::Tone,
        ] {
            for stroke in packet.strokes.iter().filter(|stroke| stroke.role == role) {
                let bytes = stroke_data_bytes(stroke);
                if lower_priority_allowed && bytes <= remaining {
                    remaining -= bytes;
                    retained.push(stroke.clone());
                } else {
                    rejected += 1;
                    rejected_tone += usize::from(role == StrokeRole::Tone);
                    rejected_form_lines += usize::from(role == StrokeRole::FormLine);
                    // A lower-priority mark must not consume the remaining
                    // bytes after a construction/feature mark did not fit.
                    // This preserves the declared line hierarchy rather than
                    // merely sorting candidates once.
                    if !matches!(role, StrokeRole::Tone | StrokeRole::FormLine) {
                        lower_priority_allowed = false;
                    }
                }
            }
        }
        if rejected == 0 {
            continue;
        }
        packet.strokes = retained;
        packet.stats.strokes = packet.strokes.len();
        packet.stats.stroke_vertices = packet
            .strokes
            .iter()
            .map(|stroke| stroke.vertices.len())
            .sum();
        packet.stats.stroke_indices = packet
            .strokes
            .iter()
            .map(|stroke| stroke.indices.len())
            .sum();
        packet.stats.stroke_data_bytes = packet.strokes.iter().map(stroke_data_bytes).sum();
        packet.stats.stroke_budget_rejected += rejected;
        packet.stats.stroke_budget_exhausted = true;
        packet.stats.hatching_rejected += rejected_tone;
        packet.stats.hatching_strokes = packet.stats.hatching_strokes.saturating_sub(rejected_tone);
        packet.stats.form_line_strokes = packet
            .stats
            .form_line_strokes
            .saturating_sub(rejected_form_lines);
    }
    commands
}

fn stroke_data_bytes(stroke: &TessellatedStroke) -> usize {
    stroke.vertices.len() * std::mem::size_of::<StrokeVertex>()
        + stroke.indices.len() * std::mem::size_of::<u32>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_render_npr::{NprDebugView, NprRenderPacket, NprRenderStats, StrokeVertex};
    use glam::{Vec2, Vec4};

    #[test]
    fn paused_sketch_reuses_geometry_while_fade_advances() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        settings.sketch_paused = true;
        renderer
            .rebuild_with_delta(&settings, [640, 360], 0.016)
            .unwrap();
        let builds = renderer.stats()["packet_builds"];
        for _ in 0..20 {
            renderer
                .rebuild_with_delta(&settings, [640, 360], 0.016)
                .unwrap();
        }
        assert_eq!(renderer.stats()["packet_builds"], builds);
        settings.camera_yaw += 1.0;
        renderer
            .rebuild_with_delta(&settings, [640, 360], 0.016)
            .unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds + 1);
    }

    #[test]
    fn sketch_geometry_rebuilds_only_on_variant_epoch() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        settings.sketch_paused = false;
        settings.motion.mode = StrokeMotionMode::RedrawContinuously;
        settings.motion.redraw_hz = 8.0;
        renderer
            .rebuild_with_delta(&settings, [640, 360], 0.01)
            .unwrap();
        let builds = renderer.stats()["packet_builds"];
        for _ in 0..10 {
            renderer
                .rebuild_with_delta(&settings, [640, 360], 0.01)
                .unwrap();
        }
        assert_eq!(renderer.stats()["packet_builds"], builds);
        renderer
            .rebuild_with_delta(&settings, [640, 360], 0.03)
            .unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds + 1);
    }

    #[test]
    fn compositor_layer_edits_reuse_extracted_paths() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        settings.sketch_paused = true;
        renderer.rebuild(&settings, [640, 360]).unwrap();
        let builds = renderer.stats()["packet_builds"];
        let layer = settings.style_layers.layer_mut("contours").unwrap();
        layer.opacity = 0.37;
        layer.blend = NprBlendMode::Multiply;
        layer.color_source = NprLayerColorSource::Constant(Vec4::new(0.2, 0.4, 0.8, 1.0));
        renderer.rebuild(&settings, [640, 360]).unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds);
        assert_eq!(
            renderer.commands()[0]
                .layers
                .layer("contours")
                .unwrap()
                .opacity,
            0.37
        );
    }

    #[test]
    fn enabling_a_line_rebuilds_missing_geometry_and_local_edits_stay_local() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        settings.style_layers.layer_mut("contours").unwrap().enabled = false;
        settings.style_layers.layer_mut("contours").unwrap().tool = Some(StrokeTool::Pencil);
        renderer.rebuild(&settings, [256, 256]).unwrap();
        let builds = renderer.stats()["packet_builds"];
        settings.style_layers.layer_mut("contours").unwrap().enabled = true;
        renderer.rebuild(&settings, [256, 256]).unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds + 1);
        let before = renderer.commands()[0].packet.clone();
        assert!(
            before
                .strokes
                .iter()
                .any(|s| s.layer_id.as_deref() == Some("contours"))
        );
        settings
            .style_layers
            .layer_mut("contours")
            .unwrap()
            .brush
            .as_mut()
            .unwrap()
            .pressure_profile = Some(0.1);
        renderer.rebuild(&settings, [256, 256]).unwrap();
        let after = renderer.commands()[0].packet.clone();
        let selected = |packet: &NprRenderPacket, id: &str| {
            packet
                .strokes
                .iter()
                .filter(|s| s.layer_id.as_deref() == Some(id))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_ne!(selected(&before, "contours"), selected(&after, "contours"));
        assert_ne!(
            selected(&before, "contours")[0].vertices[0].width,
            selected(&after, "contours")[0].vertices[0].width
        );
        assert_eq!(selected(&before, "creases"), selected(&after, "creases"));
    }

    #[test]
    fn independent_hatching_directions_coexist_with_paint() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        let hatch = settings.style_layers.layer_mut("hatching").unwrap();
        hatch.hatch = Some(NprHatchSettings {
            angle: Some(-40.0),
            cross: Some(0.0),
            ..Default::default()
        });
        let mut second = hatch.clone();
        second.id = "cross-lines".into();
        second.hatch.as_mut().unwrap().angle = Some(60.0);
        settings.style_layers.layers.push(second);
        renderer.rebuild(&settings, [256, 256]).unwrap();
        let packet = renderer.commands()[0].packet.clone();
        let paths = |id: &str| {
            packet
                .strokes
                .iter()
                .filter(|s| s.layer_id.as_deref() == Some(id))
                .flat_map(|s| s.vertices.iter().map(|v| v.position))
                .collect::<Vec<_>>()
        };
        assert!(!paths("hatching").is_empty());
        assert!(!paths("cross-lines").is_empty());
        assert_ne!(paths("hatching"), paths("cross-lines"));
        assert!(!packet.underpainting.is_empty());
        let builds = renderer.stats()["packet_builds"];
        settings
            .style_layers
            .layer_mut("hatching")
            .unwrap()
            .hatch
            .as_mut()
            .unwrap()
            .spacing = 2.0;
        renderer.rebuild(&settings, [256, 256]).unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds + 1);
    }

    #[test]
    fn paper_colour_and_geometry_targets_refresh_without_rebuilding_lines() {
        let renderer = NprPlaygroundRenderService::default();
        let mut settings = Settings::for_scene();
        renderer.rebuild(&settings, [256, 256]).unwrap();
        let builds = renderer.stats()["packet_builds"];
        let color = Vec4::new(0.2, 0.3, 0.4, 1.0);
        settings
            .style_layers
            .layer_mut("paper")
            .unwrap()
            .color_source = NprLayerColorSource::Constant(color);
        settings.style_layers.layer_mut("contours").unwrap().target =
            GeometryTarget::SurfaceFeatures {
                features: vec!["shadow-hatch".into()],
            };
        renderer.rebuild(&settings, [256, 256]).unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds);
        assert_eq!(renderer.background().unwrap().color, color.to_array());
        assert_eq!(renderer.commands()[0].packet.background, color);
        assert_eq!(
            renderer.layer_diagnostics()["contours"].no_effect_reason,
            Some(NprLayerNoEffectReason::TargetExcludesGeometry)
        );
        settings.style_layers.layer_mut("contours").unwrap().target = GeometryTarget::All;
        settings.style_layers.layer_mut("contours").unwrap().opacity = 0.0;
        renderer.rebuild(&settings, [256, 256]).unwrap();
        assert_eq!(renderer.stats()["packet_builds"], builds);
        assert_eq!(
            renderer.layer_diagnostics()["contours"].no_effect_reason,
            Some(NprLayerNoEffectReason::ZeroOpacity)
        );
    }

    #[test]
    fn layer_diagnostics_report_the_extracted_source_and_contribution() {
        let renderer = NprPlaygroundRenderService::default();
        renderer
            .rebuild(&Settings::for_scene(), [512, 512])
            .unwrap();
        let contours = &renderer.layer_diagnostics()["contours"];
        assert!(contours.source_geometry > 0);
        assert!(contours.generated_marks > 0);
        assert!(contours.extraction_micros > 0);
    }

    fn stroke(role: StrokeRole, vertices: usize) -> TessellatedStroke {
        TessellatedStroke {
            vertices: vec![
                StrokeVertex {
                    surface: None,
                    position: Vec2::ZERO,
                    width: 1.0,
                    id: 1,
                    depth: 0.5,
                    pressure: 1.0,
                    coverage: 1.0,
                    grain: 0.0,
                    edge: 1.0,
                    edge_softness: 0.0,
                    paper_tooth: 0.0,
                    dryness: 0.0,
                };
                vertices
            ],
            indices: vec![],
            id: vertices as u32,
            role,
            ..Default::default()
        }
    }

    fn command(object_id: &str, strokes: Vec<TessellatedStroke>) -> SourceCommand {
        SourceCommand {
            object_id: object_id.into(),
            temporal_scope: 0,
            command: NprDrawCommand::new(NprRenderPacket {
                occluders: vec![],
                underpainting: vec![],
                fills: vec![],
                strokes,
                background: Vec4::ONE,
                debug_view: NprDebugView::Final,
                ink: Vec4::ONE,
                stats: NprRenderStats::default(),
            }),
        }
    }

    #[test]
    fn stroke_budget_keeps_selected_source_before_other_tone() {
        let selected_feature = stroke(StrokeRole::Feature, 2);
        let selected_tone = stroke(StrokeRole::Tone, 1);
        let other_tone = stroke(StrokeRole::Tone, 1);
        let budget = stroke_data_bytes(&selected_feature) + stroke_data_bytes(&selected_tone);
        let retained = retain_strokes_with_budget(
            vec![
                command("other", vec![other_tone]),
                command("selected", vec![selected_feature, selected_tone]),
            ],
            "selected",
            budget,
        );
        assert_eq!(retained[0].object_id, "selected");
        assert_eq!(retained[0].command.packet.strokes.len(), 2);
        assert!(retained[1].command.packet.strokes.is_empty());
        assert_eq!(retained[1].command.packet.stats.stroke_budget_rejected, 1);
    }

    #[test]
    fn stroke_budget_never_replaces_a_rejected_feature_with_tone() {
        let feature = stroke(StrokeRole::Feature, 2);
        let tone = stroke(StrokeRole::Tone, 1);
        let retained = retain_strokes_with_budget(
            vec![command("selected", vec![feature, tone])],
            "selected",
            stroke_data_bytes(&stroke(StrokeRole::Tone, 1)),
        );
        assert!(retained[0].command.packet.strokes.is_empty());
        assert_eq!(retained[0].command.packet.stats.stroke_budget_rejected, 2);
    }
}

fn viewport_vec(viewport: [u32; 2]) -> glam::Vec2 {
    glam::Vec2::new(viewport[0] as f32, viewport[1] as f32)
}

fn world_camera(settings: &Settings, viewport: [u32; 2]) -> Option<PerspectiveCamera> {
    if viewport.contains(&0) {
        return None;
    }
    let yaw = settings.camera_yaw.to_radians();
    let pitch = settings.camera_pitch.to_radians();
    let position = settings.camera_target
        + Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        ) * settings.camera_distance;
    let forward = (settings.camera_target - position).normalize_or_zero();
    (forward.length_squared() > 1e-12).then_some(PerspectiveCamera {
        position,
        forward,
        up: Vec3::Y,
        vertical_fov: settings.camera_fov.to_radians(),
        near: 0.05,
        aspect: viewport[0] as f32 / viewport[1] as f32,
    })
}

fn object_transform(object: &ObjectSettings) -> Mat4 {
    let rotation = object.rotation.map(f32::to_radians);
    Mat4::from_scale_rotation_translation(
        Vec3::splat(object.scale),
        Quat::from_euler(glam::EulerRot::YXZ, rotation.y, rotation.x, rotation.z),
        object.position,
    )
}

fn object_seed(id: &str, model: &str, seed: u64) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64 ^ seed;
    for byte in id.bytes().chain(model.bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn projected_extent(geometry: &NprGeometry, camera: PerspectiveCamera, viewport: [u32; 2]) -> f32 {
    let viewport = glam::Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let mut min = glam::Vec2::splat(f32::INFINITY);
    let mut max = glam::Vec2::splat(f32::NEG_INFINITY);
    let mut count = 0usize;
    for point in geometry
        .vertices
        .iter()
        .filter_map(|vertex| camera.project(vertex.position, viewport))
    {
        min = min.min(point.screen);
        max = max.max(point.screen);
        count += 1;
    }
    (count >= 2 && min.is_finite() && max.is_finite())
        .then_some((max - min).length())
        .unwrap_or(0.0)
}

/// A small, deterministic subset of real surface vertices is enough to measure
/// projected motion. Unlike a bounding-box centre it observes object rotation.
fn projected_motion_anchors(
    geometry: &NprGeometry,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
) -> Vec<glam::Vec2> {
    let viewport = glam::Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let stride = (geometry.vertices.len() / 12).max(1);
    geometry
        .vertices
        .iter()
        .step_by(stride)
        .take(12)
        .filter_map(|vertex| {
            camera
                .project(vertex.position, viewport)
                .map(|point| point.screen)
        })
        .collect()
}

fn variant_seed(seed: u64, epoch: u32, strength: f32) -> u64 {
    if epoch == 0 || strength <= 0.0 {
        return seed;
    }
    let mut variant = seed ^ u64::from(epoch).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    variant ^= variant >> 30;
    variant = variant.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    variant ^= variant >> 27;
    variant = variant.wrapping_mul(0x94d0_49bb_1331_11eb);
    variant ^= variant >> 31;
    if strength >= 1.0 {
        return variant;
    }
    let mut mixed = seed;
    for bit in 0..64 {
        let mut selector = variant ^ (bit as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        selector ^= selector >> 29;
        selector = selector.wrapping_mul(0x3196_42b2_d24d_8ec3);
        let unit = (selector as u32 as f32) / u32::MAX as f32;
        if unit < strength {
            mixed = (mixed & !(1u64 << bit)) | (variant & (1u64 << bit));
        }
    }
    mixed
}

fn temporal_scope(id: &str, model: &str, style: ComicInk, seed: u64) -> u64 {
    // Hash only dependencies which change the *source* of a stroke. Colours,
    // paper, widths and local-space lighting modulate an existing drawing and
    // must not make every identity appear new after a panel edit or rotation.
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let source_style = format!(
        "{:?}:{:?}:{:.6}:{:.6}:{:.6}:{:.6}:{:.6}",
        style.tone_mode,
        style.surface_mode,
        style.crease_angle,
        style.smooth_crease_angle,
        style.hatching_angle,
        style.hatching_spacing,
        style.hatching_cross,
    );
    for byte in id
        .bytes()
        .chain(model.bytes())
        .chain(source_style.bytes())
        .chain(seed.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
