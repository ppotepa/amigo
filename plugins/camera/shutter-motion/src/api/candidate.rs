use amigo_plugin_api::{CandidateStatus, TargetId};

use super::{MotionShutterCoverage2d, MotionShutterResponse2d};

#[derive(Clone, Debug, PartialEq)]
pub struct MotionShutterContribution2d {
    pub owner: String,
    pub coverage: MotionShutterCoverage2d,
    pub response: MotionShutterResponse2d,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionShutterCandidate2d {
    pub owner: String,
    pub coverage: MotionShutterCoverage2d,
    pub response: MotionShutterResponse2d,
    pub status: CandidateStatus,
    pub reason: String,
    pub target_ids: Vec<TargetId>,
}

impl MotionShutterCandidate2d {
    pub fn is_active(&self) -> bool {
        self.status == CandidateStatus::Active && self.coverage.is_supported()
    }
}
