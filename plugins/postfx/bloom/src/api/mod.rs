use amigo_plugin_api::{camera_artifact_layer, scene_emissive, scene_highlight, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BloomTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl BloomTargetPlan {
    pub fn standard() -> Self {
        Self {
            reads: vec![scene_highlight(), scene_emissive()],
            writes: vec![camera_artifact_layer()],
        }
    }
}
