use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuDepthMap2dRenderExtractorBridge);
}

pub(super) fn register_installer(bridges: &WgpuRenderExtractorBridgeRegistry) {
    bridges.register(
        amigo_focus_depth_plugin::render::DEPTH_MAP_2D_EXTRACTOR_ID,
        register,
    );
}

pub struct WgpuDepthMap2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuDepthMap2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_focus_depth_plugin::render::DepthMap2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(depth_map_scene_service) =
            optional::<amigo_focus_depth_plugin::DepthMap2dSceneService>(runtime)
        else {
            return;
        };
        amigo_focus_depth_plugin::render::DepthMap2dRenderExtractor.extract(
            amigo_focus_depth_plugin::render::DepthMap2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                depth_map_scene_service: depth_map_scene_service.as_ref(),
            },
            packet,
        );
    }
}
