pub mod extraction;
pub mod primitives;

pub use extraction::*;
pub use primitives::*;

pub const LAYERED_IMAGE_2D_EXTRACTOR_ID: &str = "amigo.gfx.layered-image-2d.extractor";

pub fn register_layered_image_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(LAYERED_IMAGE_2D_EXTRACTOR_ID);
}
