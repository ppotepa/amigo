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

use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

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

pub fn register_world_2d_plugin_render_extractor_bridge_installers(
    bridges: &WgpuRenderExtractorBridgeRegistry,
) {
    tilemap::register_installer(bridges);
    sprite::register_installer(bridges);
    layered_image::register_installer(bridges);
    depth::register_installer(bridges);
    vector::register_installer(bridges);
    beacon::register_installer(bridges);
    text::register_installer(bridges);
    composition::register_installer(bridges);
    lighting::register_installer(bridges);
    particles::register_installer(bridges);
}

pub fn register_world_2d_builtin_render_extractors(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    post_fx::register(registry);
}
