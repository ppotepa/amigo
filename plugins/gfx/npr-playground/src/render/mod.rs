use crate::state::{Settings, style_preset_id};
use amigo_render_api::{NprBackgroundCommand, NprDrawCommand};
use amigo_render_npr::*;
use glam::{Mat4, Quat, Vec3};
use std::{collections::BTreeMap, path::Path, sync::Mutex};
pub const NPR_PLAYGROUND_EXTRACTOR_ID: &str = "amigo.gfx.npr-playground.extractor";
struct Prepared {
    geometry: NprGeometry,
    topology: Vec<TopologyEdge>,
}
impl Prepared {
    fn new(geometry: NprGeometry) -> Self {
        let topology = build_topology(&geometry);
        Self { geometry, topology }
    }
}

#[derive(Clone)]
struct SourceCommand {
    temporal_scope: u64,
    command: NprDrawCommand,
}

pub struct NprPlaygroundRenderService {
    geometry: Mutex<BTreeMap<String, Prepared>>,
    source: Mutex<Vec<SourceCommand>>,
    output: Mutex<(Vec<NprDrawCommand>, Option<NprBackgroundCommand>)>,
    last_input: Mutex<Option<(Settings, [u32; 2])>>,
    temporal: Mutex<DrawingHistory>,
    variants: Mutex<StrokeVariantClock>,
    lod: Mutex<BTreeMap<String, HatchLodState>>,
}
impl Default for NprPlaygroundRenderService {
    fn default() -> Self {
        Self {
            geometry: Mutex::new(
                [
                    ("cube", NprGeometry::canonical_cube()),
                    ("wedge", NprGeometry::wedge()),
                    ("cylinder", NprGeometry::cylinder(24)),
                    ("sphere", NprGeometry::icosphere()),
                ]
                .into_iter()
                .map(|(name, g)| (name.into(), Prepared::new(g)))
                .collect(),
            ),
            source: Mutex::new(vec![]),
            output: Mutex::new((vec![], None)),
            last_input: Mutex::new(None),
            temporal: Mutex::new(DrawingHistory::default()),
            variants: Mutex::new(StrokeVariantClock::default()),
            lod: Mutex::new(BTreeMap::new()),
        }
    }
}
impl NprPlaygroundRenderService {
    pub fn clear(&self) {
        *self.last_input.lock().unwrap() = None;
        self.source.lock().unwrap().clear();
        self.temporal.lock().unwrap().clear();
        self.variants.lock().unwrap().clear();
        self.lod.lock().unwrap().clear();
        *self.output.lock().unwrap() = (vec![], None);
    }
    pub fn load_models(&self, root: &Path) -> Result<(), String> {
        let mut cache = self.geometry.lock().unwrap();
        for (name, path) in [
            ("suzanne", "assets/models/suzanne/Suzanne.gltf"),
            ("avocado", "assets/models/avocado/Avocado.glb"),
        ] {
            if !cache.contains_key(name) {
                let mesh = amigo_3d_mesh::load_gltf_geometry(&root.join(path))?;
                let geometry = NprGeometry::from_indexed(&mesh.positions, &mesh.indices)?;
                cache.insert(name.into(), Prepared::new(geometry));
            }
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Option<NprDrawCommand> {
        self.output.lock().unwrap().0.first().cloned()
    }
    pub fn commands(&self) -> Vec<NprDrawCommand> {
        self.output.lock().unwrap().0.clone()
    }
    pub fn stats(&self) -> BTreeMap<String, u64> {
        let output = self.output.lock().unwrap();
        let mut stats = BTreeMap::new();
        for command in &output.0 {
            let s = &command.packet.stats;
            for (key, value) in [
                ("geometry", s.geometry),
                ("topology_edges", s.topology_edges),
                ("feature_segments", s.feature_segments),
                ("smooth_contour_spans", s.smooth_contour_spans),
                ("silhouettes", s.silhouettes),
                ("creases", s.creases),
                ("strokes", s.strokes),
                ("stroke_vertices", s.stroke_vertices),
                ("stroke_indices", s.stroke_indices),
                ("hatching_strokes", s.hatching_strokes),
                ("hatching_correction_strokes", s.hatching_correction_strokes),
                (
                    "graphite_mass_milli",
                    (s.graphite_mass.max(0.0) * 1000.0).round() as usize,
                ),
                ("hatching_candidates", s.hatching_candidates),
                ("hatching_rejected", s.hatching_rejected),
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
    /// Rebuilds a frozen reference frame. Tests and deterministic screenshots
    /// use this entry point; interactive extraction uses `rebuild_with_delta`.
    pub fn rebuild(&self, settings: &Settings, viewport: [u32; 2]) -> Result<(), String> {
        self.rebuild_internal(settings, viewport, 0.0, false)
    }

    /// Applies session-owned temporal continuity after rebuilding (only when
    /// settings/viewport changed) or reusing a pure reference packet.
    pub fn rebuild_with_delta(
        &self,
        settings: &Settings,
        viewport: [u32; 2],
        delta_seconds: f32,
    ) -> Result<(), String> {
        self.rebuild_internal(settings, viewport, delta_seconds, true)
    }

    fn rebuild_internal(
        &self,
        settings: &Settings,
        viewport: [u32; 2],
        delta_seconds: f32,
        apply_temporal: bool,
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
        if input_changed {
            settings.validate()?;
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
            let cache = self.geometry.lock().unwrap();
            let mut lod = self.lod.lock().unwrap();
            let mut variants = self.variants.lock().unwrap();
            let mut source = Vec::new();
            for (id, object) in &settings.objects {
                if !object.visible || (!settings.gallery && *id != settings.selected) {
                    continue;
                }
                let prepared = cache
                    .get(&object.model)
                    .ok_or_else(|| format!("model {} is not prepared", object.model))?;
                let rotation = object.rotation.map(f32::to_radians);
                let transform = Mat4::from_scale_rotation_translation(
                    Vec3::splat(object.scale),
                    Quat::from_euler(glam::EulerRot::YXZ, rotation.y, rotation.x, rotation.z),
                    object.position,
                );
                let world_geometry = prepared.geometry.transformed(transform);
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
                let mut style = if object.override_style {
                    object.style
                } else {
                    settings.global
                };
                style.surface_mode = object.surface_mode;
                // Scope hashes authored intent before its view/local-space
                // adaptation. In particular, object rotation changes the
                // local light vector but must not erase drawing history.
                let temporal_scope = temporal_scope(id, &object.model, style, settings.seed);
                style.light_direction = inverse
                    .transform_vector3(settings.global.light_direction)
                    .normalize_or_zero();
                let preset = style_preset_id(style);
                let variant_epoch = variants.advance(
                    temporal_scope,
                    &projected_motion_anchors(&world_geometry, world_camera, viewport),
                    delta_seconds,
                    settings.motion,
                );
                let variant_epoch = object.gesture_variant.wrapping_add(variant_epoch);
                let variant_strength = if object.gesture_variant == 0 {
                    settings.motion.redraw_strength
                } else {
                    1.0
                };
                let decision = lod.entry(id.clone()).or_default().advance(
                    projected_extent(&world_geometry, world_camera, viewport),
                    HatchLodPolicy::default(),
                );
                style.hatching_spacing *= decision.spacing_multiplier;
                let mut packet = build_packet_with_topology(
                    &prepared.geometry,
                    &prepared.topology,
                    camera,
                    viewport,
                    style,
                    variant_seed(
                        object_seed(id, &object.model, settings.seed),
                        variant_epoch,
                        variant_strength,
                    ),
                    debug,
                );
                packet.stats.hatching_lod_tier = decision.tier;
                packet.stats.gesture_variant_epoch = variant_epoch;
                if settings.gallery && settings.highlight_selected && *id == settings.selected {
                    packet.mark_selection(
                        &prepared.geometry,
                        camera,
                        glam::Vec4::new(0.15, 0.65, 0.85, 1.0),
                    );
                }
                source.push(SourceCommand {
                    temporal_scope,
                    command: NprDrawCommand::with_preset(packet, preset),
                });
            }
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
                .map(|entry| {
                    let mut command = entry.command;
                    history.advance_packet_in_frame(
                        entry.temporal_scope,
                        &mut command.packet,
                        delta_seconds,
                        policy,
                    );
                    command
                })
                .collect();
            history.finish_frame(delta_seconds, policy);
            commands
        } else {
            source.into_iter().map(|entry| entry.command).collect()
        };
        *self.output.lock().unwrap() = (
            commands,
            Some(NprBackgroundCommand {
                color: settings.global.paper.to_array(),
                grain: settings.global.paper_grain,
                tooth: settings.global.paper_tooth,
                seed: settings.seed,
            }),
        );
        Ok(())
    }
    pub fn rebuild_cube(&self, viewport: [u32; 2], seed: u64) {
        let mut settings = Settings::for_scene(false);
        settings.seed = seed;
        self.rebuild(&settings, viewport)
            .expect("built-in cube is valid");
    }
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
