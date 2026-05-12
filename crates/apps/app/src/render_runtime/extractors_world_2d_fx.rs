use super::extractors_world_2d_composition;
use super::extractors_world_2d_lighting;
use super::extractors_world_2d_particles;
use super::extractors_world_2d_postfx;
use super::context::AppRenderExtractorRegistry;

pub(crate) fn register_world_2d_fx_render_extractors<'a>(registry: &mut AppRenderExtractorRegistry<'a>) {
    extractors_world_2d_composition::register_world_2d_composition_render_extractors(registry);
    extractors_world_2d_lighting::register_world_2d_lighting_render_extractors(registry);
    extractors_world_2d_particles::register_world_2d_particles_render_extractors(registry);
    extractors_world_2d_postfx::register_world_2d_postfx_render_extractors(registry);
}
