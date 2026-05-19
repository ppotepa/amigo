pub mod runtime;

pub use runtime::*;

use amigo_plugin_api::{scene_color, CandidateStatus, TargetId};

#[derive(Clone, Debug, PartialEq)]
pub struct Material2dSource {
    pub id: String,
    pub base_color_rgba: [f32; 4],
    pub opacity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material2dCandidate {
    pub source: Material2dSource,
    pub status: CandidateStatus,
    pub target_ids: Vec<TargetId>,
}

impl Material2dCandidate {
    pub fn active(source: Material2dSource) -> Self {
        Self {
            source,
            status: CandidateStatus::Active,
            target_ids: vec![scene_color()],
        }
    }
}
