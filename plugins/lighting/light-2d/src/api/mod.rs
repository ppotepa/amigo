use amigo_plugin_api::{scene_lighting, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct Light2dSource {
    pub id: String,
    pub color_rgba: [f32; 4],
    pub intensity: f32,
    pub radius_px: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light2dCandidate {
    pub source: Light2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Light2dCandidate {
    pub fn active(source: Light2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_lighting()],
        }
    }
}
