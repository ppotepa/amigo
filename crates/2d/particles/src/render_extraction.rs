use crate::{Particle2dDrawCommand, Particle2dSceneService};

pub struct Particle2dRenderExtractionContext<'a> {
    pub particle2d_scene_service: &'a Particle2dSceneService,
}

pub fn extract_particle2d_render_commands(
    ctx: Particle2dRenderExtractionContext<'_>,
) -> Vec<Particle2dDrawCommand> {
    ctx.particle2d_scene_service.draw_commands()
}
