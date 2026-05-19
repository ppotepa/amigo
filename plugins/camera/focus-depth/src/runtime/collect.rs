use crate::api::{FocusDepthCandidate2d, FocusDepthSource2d};

pub fn collect_focus_depth_candidates_2d(
    sources: &[FocusDepthSource2d],
) -> Vec<FocusDepthCandidate2d> {
    sources
        .iter()
        .map(super::resolve_focus_depth_candidate_2d)
        .collect()
}
