use amigo_plugin_api::{final_composite, scene_color, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorGradingTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl ColorGradingTargetPlan {
    pub fn final_composite() -> Self {
        Self {
            reads: vec![scene_color()],
            writes: vec![final_composite()],
        }
    }
}
