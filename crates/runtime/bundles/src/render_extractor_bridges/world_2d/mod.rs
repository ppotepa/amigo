mod beacon;
mod camera_capture;
#[cfg(test)]
mod camera_capture_tests;
mod common;
mod composition;
mod depth;
mod layered_image;
mod lighting;
mod particles;
mod post_fx;
mod sprite;
mod text;
mod tilemap;
mod vector;

pub fn register_world_2d_render_extractors(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    register_world_2d_builtin_render_extractors(registry);
    tilemap::register(registry);
    sprite::register(registry);
    layered_image::register(registry);
    depth::register(registry);
    vector::register(registry);
    beacon::register(registry);
    text::register(registry);
    composition::register(registry);
    lighting::register(registry);
    particles::register(registry);
}

pub(crate) fn tilemap_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { tilemap::installer() }
pub(crate) fn sprite_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { sprite::installer() }
pub(crate) fn layered_image_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { layered_image::installer() }
pub(crate) fn depth_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { depth::installer() }
pub(crate) fn vector_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { vector::installer() }
pub(crate) fn beacon_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { beacon::installer() }
pub(crate) fn text_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { text::installer() }
pub(crate) fn composition_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { composition::installer() }
pub(crate) fn lighting_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { lighting::installer() }
pub(crate) fn particles_bridge_installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller { particles::installer() }

pub fn register_world_2d_builtin_render_extractors(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    post_fx::register(registry);
}
