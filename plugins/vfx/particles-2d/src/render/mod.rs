pub mod extraction;
pub mod primitives;

pub use extraction::*;
pub use primitives::*;

pub const PARTICLE_2D_EXTRACTOR_ID: &str = "amigo.vfx.particles-2d.extractor";

pub fn register_particle_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(PARTICLE_2D_EXTRACTOR_ID);
}
