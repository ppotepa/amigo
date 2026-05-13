use crate::{Particle2dDrawCommand, Particle2dSceneService};

pub struct Particle2dRenderExtractionContext<'a> {
    pub particle2d_scene_service: &'a Particle2dSceneService,
}

pub trait Particle2dRenderOutput {
    fn push_particle2d_render_command(&mut self, command: Particle2dDrawCommand);
}

pub struct Particle2dRenderExtractor;

impl Particle2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "particle_2d"
    }

    pub fn extract(
        &self,
        ctx: Particle2dRenderExtractionContext<'_>,
        output: &mut impl Particle2dRenderOutput,
    ) {
        for command in extract_particle2d_render_commands(ctx) {
            output.push_particle2d_render_command(command);
        }
    }
}

pub fn extract_particle2d_render_commands(
    ctx: Particle2dRenderExtractionContext<'_>,
) -> Vec<Particle2dDrawCommand> {
    ctx.particle2d_scene_service.draw_commands()
}

