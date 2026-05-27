use amigo_plugin_api::{CandidateStatus, TargetId, scene_alpha, scene_color};

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter2dSource {
    pub id: String,
    pub render_layer: String,
    pub intensity: f32,
    pub motion_response: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Particle2dCandidate {
    pub source: ParticleEmitter2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Particle2dCandidate {
    pub fn active(source: ParticleEmitter2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_color(), scene_alpha()],
        }
    }
}
