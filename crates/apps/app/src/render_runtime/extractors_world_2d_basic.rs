use amigo_render_api::RenderFrameExtractor;
use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};
use super::extractors_world_2d_layered_image;
use super::extractors_world_2d_sprite;
use super::extractors_world_2d_tilemap;
use super::extractors_world_2d_vector;

pub(crate) fn register_world_2d_basic_render_extractors<'a>(registry: &mut AppRenderExtractorRegistry<'a>) {
    extractors_world_2d_tilemap::register_world_2d_tilemap_render_extractors(registry);
    extractors_world_2d_sprite::register_world_2d_sprite_render_extractors(registry);
    extractors_world_2d_layered_image::register_world_2d_layered_image_render_extractors(registry);
    extractors_world_2d_vector::register_world_2d_vector_render_extractors(registry);
}
