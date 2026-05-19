use amigo_plugin_api::{scene_alpha, scene_color, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Text2dCoverage {
    Glyphs {
        entity_name: String,
        render_layer: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text2dRenderableCandidate {
    pub entity_name: String,
    pub coverage: Text2dCoverage,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Text2dRenderableCandidate {
    pub fn active(entity_name: impl Into<String>, render_layer: impl Into<String>) -> Self {
        let entity_name = entity_name.into();
        Self {
            coverage: Text2dCoverage::Glyphs {
                entity_name: entity_name.clone(),
                render_layer: render_layer.into(),
            },
            entity_name,
            status: CandidateStatus::Active,
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
