use amigo_plugin_api::{scene_alpha, scene_color, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct Tilemap2dCandidate {
    pub entity_name: String,
    pub render_layer: String,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Tilemap2dCandidate {
    pub fn active(entity_name: impl Into<String>, render_layer: impl Into<String>) -> Self {
        Self {
            entity_name: entity_name.into(),
            render_layer: render_layer.into(),
            status: CandidateStatus::Active,
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
