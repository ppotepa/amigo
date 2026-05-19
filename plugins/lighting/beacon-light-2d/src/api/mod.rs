use amigo_plugin_api::{scene_lighting, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct BeaconLight2dSource {
    pub id: String,
    pub color_rgba: [f32; 4],
    pub intensity: f32,
    pub radius_px: f32,
    pub animated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeaconLight2dCandidate {
    pub source: BeaconLight2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl BeaconLight2dCandidate {
    pub fn active(source: BeaconLight2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_lighting()],
        }
    }
}
