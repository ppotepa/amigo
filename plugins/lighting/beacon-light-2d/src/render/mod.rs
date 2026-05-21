pub mod extraction;
pub mod primitives;

pub use extraction::*;
pub use primitives::*;

pub const BEACON_2D_EXTRACTOR_ID: &str = "amigo.lighting.beacon-light-2d.extractor";

pub fn register_beacon_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(BEACON_2D_EXTRACTOR_ID);
}
