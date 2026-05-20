use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuParticle2dRenderExtractorBridge);
}

pub struct WgpuParticle2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuParticle2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_particles_2d_plugin::Particle2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(particle2d_scene_service) =
            optional::<amigo_particles_2d_plugin::Particle2dSceneService>(runtime)
        else {
            return;
        };
        amigo_particles_2d_plugin::Particle2dRenderExtractor.extract(
            amigo_particles_2d_plugin::Particle2dRenderExtractionContext {
                particle2d_scene_service: particle2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}
