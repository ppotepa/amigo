use crate::ids::{DomainId, TargetId};
use crate::status::CandidateStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateTrace {
    pub domain: DomainId,
    pub status: CandidateStatus,
    pub reason: Option<String>,
    pub targets: Vec<TargetId>,
}

pub trait DomainCandidate {
    fn domain(&self) -> DomainId;
    fn status(&self) -> CandidateStatus;
    fn target_ids(&self) -> &[TargetId];
    fn trace(&self) -> Option<&CandidateTrace>;
}
