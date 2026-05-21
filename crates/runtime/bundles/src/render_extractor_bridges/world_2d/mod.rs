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
    register_world_2d_plugin_tilemap_extractor(registry);
    register_world_2d_plugin_sprite_extractor(registry);
    register_world_2d_plugin_layered_image_extractor(registry);
    register_world_2d_plugin_depth_map_extractor(registry);
    register_world_2d_plugin_vector_extractor(registry);
    register_world_2d_plugin_beacon_extractor(registry);
    register_world_2d_plugin_text_extractor(registry);
    register_world_2d_plugin_composition_extractor(registry);
    register_world_2d_plugin_lighting_extractor(registry);
    register_world_2d_plugin_particles_extractor(registry);
}

pub fn register_world_2d_builtin_render_extractors(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    post_fx::register(registry);
}

pub fn register_world_2d_plugin_tilemap_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    tilemap::register(registry);
}

pub fn register_world_2d_plugin_sprite_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    sprite::register(registry);
}

pub fn register_world_2d_plugin_layered_image_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    layered_image::register(registry);
}

pub fn register_world_2d_plugin_depth_map_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    depth::register(registry);
}

pub fn register_world_2d_plugin_text_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    text::register(registry);
}

pub fn register_world_2d_plugin_vector_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    vector::register(registry);
}

pub fn register_world_2d_plugin_beacon_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    beacon::register(registry);
}

pub fn register_world_2d_plugin_composition_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    composition::register(registry);
}

pub fn register_world_2d_plugin_lighting_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    lighting::register(registry);
}

pub fn register_world_2d_plugin_particles_extractor(
    registry: &mut crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry,
) {
    particles::register(registry);
}
