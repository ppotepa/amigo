use amigo_render_api::{NprBackgroundCommand, NprDrawCommand};
use amigo_render_npr::{ComicInk, NprCamera, NprDebugView, NprGeometry, NprMaterialMotionMode, NprMotionPolicy, NprPathMotionMode, NprTemporalState, PerspectiveCamera, build_packet, build_packet_with_camera, build_pencil_animation_packet_with_camera_and_temporal};
use glam::{Mat3, Vec3, Vec4};
use std::{fs, path::PathBuf, sync::Mutex};

pub const NPR_PLAYGROUND_EXTRACTOR_ID: &str = "amigo.gfx.npr-playground.extractor";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprPlaygroundRenderProfile {
    MinimalInk,
    PencilAnimation,
}

impl NprPlaygroundRenderProfile {
    /// Authored profile data stays at the playground/domain boundary. The NPR
    /// core only receives drawing intent, never a scene-name special case.
    fn style(self) -> ComicInk {
        match self {
            Self::MinimalInk => ComicInk::default(),
            Self::PencilAnimation => ComicInk {
                paper: Vec4::new(0.94, 0.925, 0.88, 1.0),
                // These are transparent graphite masses over paper, not lit
                // material colours. The graphite stroke shader supplies the
                // small-scale tooth/deposition response.
                shadow: Vec4::new(0.24, 0.225, 0.20, 0.38),
                mid: Vec4::new(0.34, 0.32, 0.28, 0.16),
                light: Vec4::new(0.55, 0.52, 0.46, 0.0),
                outline_width: 2.1,
                crease_width: 1.35,
                boundary_width: 1.8,
                taper: 0.28,
                wobble: 0.12,
            },
        }
    }

    fn temporal(self, seconds: f32) -> NprTemporalState {
        match self {
            Self::MinimalInk => NprTemporalState::default(),
            Self::PencilAnimation => NprTemporalState::at_time(
                NprMotionPolicy {
                    // The playground is specifically a pencil-animation
                    // study: redraw gesture paths each visible frame while
                    // paper deposition evolves more slowly.
                    path_mode: NprPathMotionMode::RedrawContinuously,
                    path_redraw_hz: 60.0,
                    material_mode: NprMaterialMotionMode::Boil,
                    material_redraw_hz: 12.0,
                },
                seconds,
                true,
            ),
        }
    }
}

#[derive(Debug, Default)]
pub struct NprPlaygroundRenderService {
    command: Mutex<Option<NprDrawCommand>>,
    background: Mutex<Option<NprBackgroundCommand>>,
    suzanne: Mutex<Option<NprGeometry>>,
    suzanne_path: PathBuf,
}

impl NprPlaygroundRenderService {
    pub fn with_suzanne_path(suzanne_path: PathBuf) -> Self {
        Self {
            command: Mutex::default(),
            background: Mutex::default(),
            suzanne: Mutex::default(),
            suzanne_path,
        }
    }
    pub fn set_command(&self, command: NprDrawCommand) {
        *self.command.lock().expect("NPR command mutex") = Some(command);
    }
    pub fn snapshot(&self) -> Option<NprDrawCommand> {
        self.command.lock().expect("NPR command mutex").clone()
    }
    pub fn background(&self) -> Option<NprBackgroundCommand> {
        *self.background.lock().expect("NPR background mutex")
    }
    pub fn rebuild_cube(&self, viewport: [u32; 2], seed: u64) {
        self.rebuild_cube_rotated(viewport, seed, 0.36, 0.71);
    }
    pub fn rebuild_cube_rotated(
        &self,
        viewport: [u32; 2],
        seed: u64,
        rotation_x: f32,
        rotation_y: f32,
    ) {
        let camera =
            PerspectiveCamera::cube_default(viewport[0].max(1) as f32 / viewport[1].max(1) as f32);
        let mut geometry = NprGeometry::canonical_cube();
        let rotation = Mat3::from_rotation_y(rotation_y) * Mat3::from_rotation_x(rotation_x);
        for vertex in &mut geometry.vertices {
            vertex.position = rotation * Vec3::from(vertex.position);
        }
        let packet = build_packet(
            &geometry,
            camera,
            viewport,
            ComicInk::default(),
            seed,
            NprDebugView::Final,
        );
        let background = NprBackgroundCommand {
            color: packet.background.to_array(),
        };
        self.set_command(NprDrawCommand::new(packet));
        *self.background.lock().expect("NPR background mutex") = Some(background);
    }

    pub fn rebuild_suzanne_rotated(
        &self,
        viewport: [u32; 2],
        seed: u64,
        rotation_x: f32,
        rotation_y: f32,
        position: Vec3,
        camera: NprCamera,
        profile: NprPlaygroundRenderProfile,
        elapsed_seconds: f32,
    ) -> Result<(), String> {
        let mut geometry = {
            let mut cached = self.suzanne.lock().expect("NPR Suzanne mutex");
            if cached.is_none() {
                // `gltf::import` also loads referenced image files. The first
                // NPR exercise consumes only indexed geometry, so load buffer
                // data directly and deliberately leave material textures out.
                let document = gltf::Gltf::open(&self.suzanne_path)
                    .map_err(|error| error.to_string())?;
                let asset_directory = self.suzanne_path.parent().ok_or_else(|| {
                    "Suzanne asset path has no parent directory".to_owned()
                })?;
                let buffers = document
                    .buffers()
                    .map(|buffer| match buffer.source() {
                        gltf::buffer::Source::Uri(uri) => fs::read(asset_directory.join(uri))
                            .map_err(|error| format!("could not read Suzanne buffer `{uri}`: {error}")),
                        gltf::buffer::Source::Bin => document
                            .blob
                            .clone()
                            .ok_or_else(|| "Suzanne GLB buffer is missing".to_owned()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut positions = Vec::new();
                let mut indices = Vec::new();
                for mesh in document.meshes() {
                    for primitive in mesh.primitives() {
                        let reader = primitive.reader(|buffer| {
                            buffers.get(buffer.index()).map(Vec::as_slice)
                        });
                        let Some(source_positions) = reader.read_positions() else { continue };
                        let base = positions.len() as u32;
                        positions.extend(source_positions);
                        if let Some(source_indices) = reader.read_indices() {
                            indices.extend(source_indices.into_u32().map(|index| base + index));
                        }
                    }
                }
                *cached = Some(NprGeometry::from_indexed(&positions, &indices)?);
            }
            cached.as_ref().expect("Suzanne geometry initialized").clone()
        };
        let rotation = Mat3::from_rotation_y(rotation_y) * Mat3::from_rotation_x(rotation_x);
        for vertex in &mut geometry.vertices {
            vertex.position = rotation * Vec3::from(vertex.position) + position;
        }
        let packet = match profile {
            NprPlaygroundRenderProfile::MinimalInk => build_packet_with_camera(
                &geometry, camera, viewport, profile.style(), seed, NprDebugView::Final,
            ),
            NprPlaygroundRenderProfile::PencilAnimation => build_pencil_animation_packet_with_camera_and_temporal(
                &geometry,
                camera,
                viewport,
                profile.style(),
                seed,
                NprDebugView::Final,
                profile.temporal(elapsed_seconds),
            ),
        };
        let background = NprBackgroundCommand { color: packet.background.to_array() };
        self.set_command(NprDrawCommand::new(packet));
        *self.background.lock().expect("NPR background mutex") = Some(background);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suzanne_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../mods/npr-playground/assets/models/suzanne/Suzanne.gltf")
    }

    #[test]
    fn pencil_suzanne_is_not_a_topology_wireframe() {
        let service = NprPlaygroundRenderService::with_suzanne_path(suzanne_path());
        service.rebuild_suzanne_rotated(
            [1280, 720],
            0x4E5052,
            0.0,
            0.0,
            Vec3::ZERO,
            PerspectiveCamera::cube_default(1280.0 / 720.0).into(),
            NprPlaygroundRenderProfile::PencilAnimation,
            0.0,
        ).unwrap();
        let packet = service.snapshot().expect("Pencil packet").packet;

        assert!(packet.stats.topology_edges > 0);
        assert!(packet.strokes.len() * 4 < packet.stats.topology_edges as usize,
            "pencil generated {} strokes from {} topology edges", packet.strokes.len(), packet.stats.topology_edges);
        assert!(packet.fills.is_empty());
    }

    #[test]
    fn pencil_suzanne_redraws_the_hand_path_each_animation_frame() {
        let service = NprPlaygroundRenderService::with_suzanne_path(suzanne_path());
        let camera = PerspectiveCamera::cube_default(1280.0 / 720.0).into();
        service.rebuild_suzanne_rotated(
            [1280, 720], 0x4E5052, 0.0, 0.0, Vec3::ZERO, camera,
            NprPlaygroundRenderProfile::PencilAnimation, 0.0,
        ).unwrap();
        let first = service.snapshot().expect("first pencil packet").packet;
        service.rebuild_suzanne_rotated(
            [1280, 720], 0x4E5052, 0.0, 0.0, Vec3::ZERO, camera,
            NprPlaygroundRenderProfile::PencilAnimation, 0.034,
        ).unwrap();
        let second = service.snapshot().expect("second pencil packet").packet;

        assert!(!first.strokes.is_empty());
        assert_ne!(first.strokes, second.strokes);
    }
}
