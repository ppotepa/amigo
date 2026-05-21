pub mod extraction;
pub mod primitives;

use amigo_plugin_api::{scene_alpha, scene_color, TargetId};

pub use extraction::*;
pub use primitives::*;

pub const VECTOR_2D_EXTRACTOR_ID: &str = "amigo.gfx.vector-2d.extractor";
pub const VECTOR_2D_RENDERABLE_KIND: &str = "amigo.gfx.vector-2d.renderable";

pub fn register_vector_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(VECTOR_2D_EXTRACTOR_ID);
}

pub fn vector_render_targets() -> Vec<TargetId> {
    vec![scene_color(), scene_alpha()]
}
