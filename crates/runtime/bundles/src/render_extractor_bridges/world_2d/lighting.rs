use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use super::common::optional;
use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuLighting2dRenderExtractorBridge);
}

pub(super) fn installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
    crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
        extractor_id: amigo_light_2d_plugin::LIGHTING_2D_EXTRACTOR_ID,
        install: register,
    }
}

pub struct WgpuLighting2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuLighting2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_light_2d_plugin::Lighting2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(global_light2d_scene_service) =
            optional::<amigo_light_2d_plugin::GlobalLight2dSceneService>(runtime)
        else {
            return;
        };
        let Some(lightmap2d_scene_service) =
            optional::<amigo_light_2d_plugin::LightMap2dSceneService>(runtime)
        else {
            return;
        };
        let Some(light_group2d_scene_service) =
            optional::<amigo_light_2d_plugin::LightGroup2dSceneService>(runtime)
        else {
            return;
        };
        let commands = amigo_light_2d_plugin::extract_lighting2d_render_commands(
            amigo_light_2d_plugin::Lighting2dRenderExtractionContext {
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                lightmap2d_scene_service: lightmap2d_scene_service.as_ref(),
                light_group2d_scene_service: light_group2d_scene_service.as_ref(),
            },
        );
        for command in &commands.global_lights {
            packet.push_render_contribution_2d(
                amigo_light_2d_plugin::global_light_command_to_render_contribution(command),
            );
        }
        for contribution in
            amigo_light_2d_plugin::lightmap_commands_to_render_contributions(&commands.lightmaps)
        {
            packet.push_render_contribution_2d(contribution);
        }
        for contribution in amigo_light_2d_plugin::light_group_commands_to_render_contributions(
            &commands.light_groups,
            &commands.global_lights,
            &commands.lightmaps,
        ) {
            packet.push_render_contribution_2d(contribution);
        }
    }
}
