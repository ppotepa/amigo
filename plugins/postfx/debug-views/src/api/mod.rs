use amigo_plugin_api::{diagnostics_snapshot, scene_color, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugViewTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl DebugViewTargetPlan {
    pub fn standard() -> Self {
        Self {
            reads: vec![scene_color()],
            writes: vec![diagnostics_snapshot()],
        }
    }
}
