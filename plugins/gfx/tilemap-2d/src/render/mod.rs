pub mod extraction;
pub mod primitives;

pub use extraction::*;
pub use primitives::*;

pub const TILEMAP_2D_EXTRACTOR_ID: &str = "amigo.gfx.tilemap-2d.extractor";

pub fn register_tilemap_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(TILEMAP_2D_EXTRACTOR_ID);
}
