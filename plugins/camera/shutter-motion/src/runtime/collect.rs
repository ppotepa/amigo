use crate::api::{MotionShutterCandidate2d, MotionShutterSource2d};

pub fn collect_motion_shutter_candidates_2d(
    sources: &[MotionShutterSource2d],
) -> Vec<MotionShutterCandidate2d> {
    sources
        .iter()
        .map(super::resolve_motion_shutter_candidate_2d)
        .collect()
}
