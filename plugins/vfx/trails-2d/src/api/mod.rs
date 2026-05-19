use amigo_plugin_api::{scene_alpha, scene_color, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct Trail2dSource {
    pub id: String,
    pub render_layer: String,
    pub length_px: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trail2dCandidate {
    pub source: Trail2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Trail2dCandidate {
    pub fn active(source: Trail2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
