use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_particles_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedParticle2dExtractor);
}

pub(crate) struct ResolvedParticle2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedParticle2dExtractor {
    fn name(&self) -> &'static str { "resolved_particle_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_particles::extract_particle2d_render_commands(
            amigo_2d_particles::Particle2dRenderExtractionContext {
                particle2d_scene_service: context.particle2d_scene_service,
            },
        ) { packet.push_world_2d_particle(command); }
    }
}
