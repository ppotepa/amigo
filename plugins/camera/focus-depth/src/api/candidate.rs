use amigo_plugin_api::{CandidateStatus, TargetId};

use super::{FocusDepthCoverage2d, FocusDepthResponse2d};

#[derive(Clone, Debug, PartialEq)]
pub struct FocusDepthCandidate2d {
    pub owner: String,
    pub coverage: FocusDepthCoverage2d,
    pub response: FocusDepthResponse2d,
    pub status: CandidateStatus,
    pub reason: String,
    pub target_ids: Vec<TargetId>,
}

impl FocusDepthCandidate2d {
    pub fn is_active(&self) -> bool {
        self.status == CandidateStatus::Active && self.coverage.is_supported()
    }
}
