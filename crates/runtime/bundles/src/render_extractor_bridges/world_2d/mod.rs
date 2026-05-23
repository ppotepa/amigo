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

pub fn available_world_2d_plugin_bridge_installers(
) -> Vec<crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller> {
    vec![
        tilemap::installer(),
        sprite::installer(),
        layered_image::installer(),
        depth::installer(),
        vector::installer(),
        beacon::installer(),
        text::installer(),
        composition::installer(),
        lighting::installer(),
        particles::installer(),
    ]
}

pub fn register_world_2d_builtin_render_extractors(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    post_fx::register(registry);
}
