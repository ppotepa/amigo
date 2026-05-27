pub mod candidate_buffers;
pub mod color_coverage;
pub mod lightmap_channel;
pub mod pass;
pub mod targets;

pub use amigo_render_api::{
    scene_emissive_target_id, scene_highlight_target_id, target_id_for_visual_kind_name,
    CameraOpticalRenderTargetPlan,
};
pub use candidate_buffers::*;
pub use color_coverage::*;
pub use lightmap_channel::*;
pub use targets::*;
