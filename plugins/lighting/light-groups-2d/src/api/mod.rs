use amigo_plugin_api::{scene_lighting, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct LightGroup2dSource {
    pub id: String,
    pub lightmap_source: Option<String>,
    pub lightmap_channel: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LightGroup2dCandidate {
    pub source: LightGroup2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl LightGroup2dCandidate {
    pub fn active(source: LightGroup2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_lighting()],
        }
    }
}
