pub mod extraction;
pub mod primitives;
pub mod targets;

pub use extraction::*;
pub use primitives::*;
pub use targets::*;

pub const SPRITE_2D_EXTRACTOR_ID: &str = "amigo.gfx.sprite-2d.extractor";
pub const SPRITE_2D_RENDERABLE_KIND: &str = "amigo.gfx.sprite-2d.renderable";

pub fn register_sprite_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(SPRITE_2D_EXTRACTOR_ID);
}
