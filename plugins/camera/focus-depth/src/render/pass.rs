use amigo_plugin_api::TargetId;

use crate::api::{focus_field_target_id, scene_depth_target_id};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusDepthRenderTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl FocusDepthRenderTargetPlan {
    pub fn focus_field() -> Self {
        Self {
            reads: vec![scene_depth_target_id()],
            writes: vec![focus_field_target_id()],
        }
    }
}
