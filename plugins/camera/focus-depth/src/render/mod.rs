pub mod extraction;
pub mod pass;

pub use extraction::*;
pub use pass::*;

pub const DEPTH_MAP_2D_EXTRACTOR_ID: &str = "amigo.camera.focus-depth.extractor";

pub fn register_depth_map_2d_render_extractor_id(
    registry: &amigo_render_api::RuntimeRenderExtractorIdRegistry,
) {
    registry.register(DEPTH_MAP_2D_EXTRACTOR_ID);
}
