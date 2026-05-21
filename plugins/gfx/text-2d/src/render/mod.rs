pub mod extraction;
pub mod primitives;

use amigo_plugin_api::{scene_alpha, scene_color, TargetId};

pub use extraction::*;
pub use primitives::*;

pub const TEXT_2D_EXTRACTOR_ID: &str = "amigo.gfx.text-2d.extractor";
pub const TEXT_2D_RENDERABLE_KIND: &str = "amigo.gfx.text-2d.renderable";

pub fn register_text_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(TEXT_2D_EXTRACTOR_ID);
}

pub fn text_render_targets() -> Vec<TargetId> {
    vec![scene_color(), scene_alpha()]
}
