use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuTileMap2dRenderExtractorBridge);
}

pub struct WgpuTileMap2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuTileMap2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_tilemap_2d_plugin::TileMap2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(tilemap_scene_service) =
            optional::<amigo_tilemap_2d_plugin::TileMap2dSceneService>(runtime)
        else {
            return;
        };
        amigo_tilemap_2d_plugin::TileMap2dRenderExtractor.extract(
            amigo_tilemap_2d_plugin::TileMap2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                tilemap_scene_service: tilemap_scene_service.as_ref(),
            },
            packet,
        );
    }
}
