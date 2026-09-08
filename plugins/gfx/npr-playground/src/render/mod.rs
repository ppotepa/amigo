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
pub struct NprPlaygroundRenderService {
    geometry: Mutex<BTreeMap<String, Prepared>>,
    output: Mutex<(Vec<NprDrawCommand>, Option<NprBackgroundCommand>)>,
    last_input: Mutex<Option<(Settings, [u32; 2])>>,
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
            output: Mutex::new((vec![], None)),
            last_input: Mutex::new(None),
        }
    }
}
impl NprPlaygroundRenderService {
    pub fn clear(&self) {
        *self.last_input.lock().unwrap() = None;
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
                ("silhouettes", s.silhouettes),
                ("creases", s.creases),
                ("strokes", s.strokes),
                ("stroke_vertices", s.stroke_vertices),
                ("stroke_indices", s.stroke_indices),
            ] {
                *stats.entry(key.into()).or_insert(0) += value as u64;
            }
            stats.insert("viewport_width".into(), s.viewport[0] as u64);
            stats.insert("viewport_height".into(), s.viewport[1] as u64);
        }
        stats
    }
    pub fn background(&self) -> Option<NprBackgroundCommand> {
        self.output.lock().unwrap().1
    }
    pub fn rebuild(&self, settings: &Settings, viewport: [u32; 2]) -> Result<(), String> {
        if viewport.contains(&0) {
            self.clear();
            return Ok(());
        }
        let mut last_input = self.last_input.lock().unwrap();
        if last_input
            .as_ref()
            .is_some_and(|(last, size)| last == settings && *size == viewport)
        {
            return Ok(());
        }
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
        let debug = match settings.debug.as_str() {
            "FeatureClasses" => NprDebugView::FeatureClasses,
            "StrokeIds" => NprDebugView::StrokeIds,
            _ => NprDebugView::Final,
        };
        let cache = self.geometry.lock().unwrap();
        let mut commands = Vec::new();
        for (index, (id, object)) in settings.objects.iter().enumerate() {
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
            let geometry = prepared.geometry.transformed(transform);
            let mut style = if object.override_style {
                object.style
            } else {
                settings.global
            };
            style.light_direction = settings.global.light_direction;
            let mut packet = build_packet_with_topology(
                &geometry,
                &prepared.topology,
                camera,
                viewport,
                style,
                settings.seed.wrapping_add(index as u64 * 997),
                debug,
            );
            if settings.gallery && settings.highlight_selected && *id == settings.selected {
                packet.mark_selection(&geometry, camera, glam::Vec4::new(0.15, 0.65, 0.85, 1.0));
            }
            commands.push(NprDrawCommand::with_preset(packet, style_preset_id(style)));
        }
        *self.output.lock().unwrap() = (
            commands,
            Some(NprBackgroundCommand {
                color: settings.global.paper.to_array(),
                grain: settings.global.paper_grain,
                tooth: settings.global.paper_tooth,
                seed: settings.seed,
            }),
        );
        *last_input = Some((settings.clone(), viewport));
        Ok(())
    }
    pub fn rebuild_cube(&self, viewport: [u32; 2], seed: u64) {
        let mut settings = Settings::for_scene(false);
        settings.seed = seed;
        self.rebuild(&settings, viewport)
            .expect("built-in cube is valid");
    }
}
