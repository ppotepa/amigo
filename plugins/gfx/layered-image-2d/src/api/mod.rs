use amigo_plugin_api::{scene_alpha, scene_color, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct LayeredImage2dLayer {
    pub id: String,
    pub distance_m: Option<f32>,
    pub blur_scale: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayeredImage2dCandidate {
    pub entity_name: String,
    pub layers: Vec<LayeredImage2dLayer>,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl LayeredImage2dCandidate {
    pub fn active(entity_name: impl Into<String>, layers: Vec<LayeredImage2dLayer>) -> Self {
        Self {
            entity_name: entity_name.into(),
            layers,
            status: CandidateStatus::Active,
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
