use amigo_plugin_api::TargetId;

use crate::api::{scene_velocity_target_id, temporal_exposure_target_id};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionShutterRenderTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl MotionShutterRenderTargetPlan {
    pub fn temporal_exposure() -> Self {
        Self {
            reads: vec![scene_velocity_target_id()],
            writes: vec![temporal_exposure_target_id()],
        }
    }
}
