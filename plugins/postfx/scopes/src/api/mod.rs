use amigo_plugin_api::{diagnostics_snapshot, final_composite, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopesTargetPlan {
    pub reads: Vec<TargetId>,
    pub writes: Vec<TargetId>,
}

impl ScopesTargetPlan {
    pub fn diagnostics() -> Self {
        Self {
            reads: vec![final_composite()],
            writes: vec![diagnostics_snapshot()],
        }
    }
}
