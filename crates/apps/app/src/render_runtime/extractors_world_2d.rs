use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};
use super::extractors_world_2d_basic;
use super::extractors_world_2d_fx;
use super::extractors_world_2d_text;

pub(crate) fn register_world_2d_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    extractors_world_2d_basic::register_world_2d_basic_render_extractors(registry);
    extractors_world_2d_text::register_world_2d_text_render_extractors(registry);
    extractors_world_2d_fx::register_world_2d_fx_render_extractors(registry);
}
