use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use super::common::optional;
use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuParticle2dRenderExtractorBridge);
}

pub(super) fn installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
    crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
        extractor_id: amigo_particles_2d_plugin::render::PARTICLE_2D_EXTRACTOR_ID,
        install: register,
    }
}

pub struct WgpuParticle2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuParticle2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_particles_2d_plugin::render::Particle2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(particle2d_scene_service) =
            optional::<amigo_particles_2d_plugin::Particle2dSceneService>(runtime)
        else {
            return;
        };
        for command in amigo_particles_2d_plugin::render::extract_particle2d_render_commands(
            amigo_particles_2d_plugin::render::Particle2dRenderExtractionContext {
                particle2d_scene_service: particle2d_scene_service.as_ref(),
            },
        ) {
            packet.push_renderable_2d(
                amigo_particles_2d_plugin::render::particle_draw_command_to_renderable_2d(&command),
            );
        }
    }
}
