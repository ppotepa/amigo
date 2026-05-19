use amigo_plugin_api::{camera_artifact_layer, final_composite, scene_color, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositeTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl CompositeTargetPlan {
    pub fn standard() -> Self {
        Self {
            reads: vec![scene_color(), camera_artifact_layer()],
            writes: vec![final_composite()],
        }
    }
}
