use super::context::AppRenderExtractorRegistry;
use super::extractors_host_overlay;
use super::extractors_world_2d;
use super::extractors_world_3d;

pub(crate) use extractors_host_overlay::register_host_render_extractor_provider;

pub(crate) fn default_app_render_extractor_registry<'a>() -> AppRenderExtractorRegistry<'a> {
    let mut registry = AppRenderExtractorRegistry::new();
    extractors_world_2d::register_world_2d_render_extractors(&mut registry);
    extractors_world_3d::register_world_3d_render_extractors(&mut registry);
    extractors_host_overlay::register_host_overlay_render_extractors(&mut registry);
    registry
}
