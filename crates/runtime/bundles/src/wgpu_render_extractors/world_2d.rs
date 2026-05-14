use std::sync::Arc;

use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::context::WgpuRenderExtractorRegistry;

pub fn register_world_2d_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuTileMap2dRenderExtractorBridge);
    registry.register(WgpuSprite2dRenderExtractorBridge);
    registry.register(WgpuLayeredImage2dRenderExtractorBridge);
    registry.register(WgpuVector2dRenderExtractorBridge);
    registry.register(WgpuText2dRenderExtractorBridge);
    registry.register(WgpuComposition2dRenderExtractorBridge);
    registry.register(WgpuLighting2dRenderExtractorBridge);
    registry.register(WgpuParticle2dRenderExtractorBridge);
    registry.register(WgpuPostFx2dRenderExtractorBridge);
}

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> Arc<T> {
    runtime
        .required::<T>()
        .expect("render extractor required service should be registered")
}

pub struct WgpuTileMap2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuTileMap2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_tilemap::TileMap2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let tilemap_scene_service = required::<amigo_2d_tilemap::TileMap2dSceneService>(runtime);
        amigo_2d_tilemap::TileMap2dRenderExtractor.extract(
            amigo_2d_tilemap::TileMap2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                tilemap_scene_service: tilemap_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuSprite2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuSprite2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_sprite::Sprite2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let sprite_scene_service = required::<amigo_2d_sprite::SpriteSceneService>(runtime);
        amigo_2d_sprite::Sprite2dRenderExtractor.extract(
            amigo_2d_sprite::Sprite2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                sprite_scene_service: sprite_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuLayeredImage2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuLayeredImage2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_2d_layered_image::LayeredImage2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let layered_image_scene_service =
            required::<amigo_2d_layered_image::LayeredImageSceneService>(runtime);
        amigo_2d_layered_image::LayeredImage2dRenderExtractor.extract(
            amigo_2d_layered_image::LayeredImage2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                layered_image_scene_service: layered_image_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuVector2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuVector2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_vector::Vector2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let vector_scene_service = required::<amigo_2d_vector::VectorSceneService>(runtime);
        amigo_2d_vector::Vector2dRenderExtractor.extract(
            amigo_2d_vector::Vector2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                vector_scene_service: vector_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuText2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuText2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_text::Text2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let text_scene_service = required::<amigo_2d_text::Text2dSceneService>(runtime);
        amigo_2d_text::Text2dRenderExtractor.extract(
            amigo_2d_text::Text2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                text_scene_service: text_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuComposition2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuComposition2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_2d_composition::Composition2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let render_layer2d_scene_service =
            required::<amigo_2d_composition::RenderLayer2dSceneService>(runtime);
        let light_route2d_scene_service =
            required::<amigo_2d_composition::LightRoute2dSceneService>(runtime);
        amigo_2d_composition::Composition2dRenderExtractor.extract(
            amigo_2d_composition::Composition2dRenderExtractionContext {
                render_layer2d_scene_service: render_layer2d_scene_service.as_ref(),
                light_route2d_scene_service: light_route2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuLighting2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuLighting2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_lighting::Lighting2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let global_light2d_scene_service =
            required::<amigo_2d_lighting::GlobalLight2dSceneService>(runtime);
        let lightmap2d_scene_service =
            required::<amigo_2d_lighting::LightMap2dSceneService>(runtime);
        let light_group2d_scene_service =
            required::<amigo_2d_lighting::LightGroup2dSceneService>(runtime);
        amigo_2d_lighting::Lighting2dRenderExtractor.extract(
            amigo_2d_lighting::Lighting2dRenderExtractionContext {
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                lightmap2d_scene_service: lightmap2d_scene_service.as_ref(),
                light_group2d_scene_service: light_group2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuParticle2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuParticle2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_particles::Particle2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let particle2d_scene_service =
            required::<amigo_2d_particles::Particle2dSceneService>(runtime);
        amigo_2d_particles::Particle2dRenderExtractor.extract(
            amigo_2d_particles::Particle2dRenderExtractionContext {
                particle2d_scene_service: particle2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuPostFx2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuPostFx2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_2d_post_fx::PostFx2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let post_fx_service = required::<amigo_2d_post_fx::PostFx2dService>(runtime);
        amigo_2d_post_fx::PostFx2dRenderExtractor.extract(
            amigo_2d_post_fx::PostFx2dRenderExtractionContext {
                post_fx_service: post_fx_service.as_ref(),
            },
            packet,
        );
    }
}
