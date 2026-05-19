use amigo_plugin_api::{scene_alpha, scene_color, CandidateStatus, TargetId};

use super::{Sprite2dCoverage, Sprite2dRenderResponse};

#[derive(Clone, Debug, PartialEq)]
pub struct Sprite2dRenderableCandidate {
    pub entity_name: String,
    pub coverage: Sprite2dCoverage,
    pub response: Sprite2dRenderResponse,
    pub status: CandidateStatus,
    pub reason: String,
    pub target_ids: Vec<TargetId>,
}

impl Sprite2dRenderableCandidate {
    pub fn active(
        entity_name: impl Into<String>,
        render_layer: impl Into<String>,
        response: Sprite2dRenderResponse,
    ) -> Self {
        let entity_name = entity_name.into();
        Self {
            coverage: Sprite2dCoverage::TextureAlpha {
                entity_name: entity_name.clone(),
                render_layer: render_layer.into(),
            },
            entity_name,
            response: response.normalized(),
            status: CandidateStatus::Active,
            reason: "sprite_2d_candidate_active".to_owned(),
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
