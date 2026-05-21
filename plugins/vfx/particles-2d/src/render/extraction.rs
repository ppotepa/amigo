use amigo_render_api::RenderExtractionOutput2d;

use crate::{Particle2dDrawCommand, Particle2dSceneService};

use super::PARTICLE_2D_EXTRACTOR_ID;

pub struct Particle2dRenderExtractionContext<'a> {
    pub particle2d_scene_service: &'a Particle2dSceneService,
}

pub struct Particle2dRenderExtractor;

impl Particle2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        PARTICLE_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Particle2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_particle2d_render_commands(ctx) {
            output.push_renderable_2d(super::particle_draw_command_to_renderable_2d(&command));
            if let Some(contribution) = super::particle_draw_command_to_light_contribution(&command)
            {
                output.push_render_contribution_2d(contribution);
            }
        }
    }
}

pub fn extract_particle2d_render_commands(
    ctx: Particle2dRenderExtractionContext<'_>,
) -> Vec<Particle2dDrawCommand> {
    ctx.particle2d_scene_service.draw_commands()
}
