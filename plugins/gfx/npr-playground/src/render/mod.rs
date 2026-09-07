use amigo_render_api::{NprBackgroundCommand, NprDrawCommand};
use amigo_render_npr::{ComicInk, NprDebugView, NprGeometry, PerspectiveCamera, build_packet};
use glam::{Mat3, Vec3};
use std::sync::Mutex;

pub const NPR_PLAYGROUND_EXTRACTOR_ID: &str = "amigo.gfx.npr-playground.extractor";

#[derive(Debug, Default)]
pub struct NprPlaygroundRenderService {
    command: Mutex<Option<NprDrawCommand>>,
    background: Mutex<Option<NprBackgroundCommand>>,
}

impl NprPlaygroundRenderService {
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
}
