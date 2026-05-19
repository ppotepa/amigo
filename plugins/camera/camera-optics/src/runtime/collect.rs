use crate::api::{CameraOpticalCandidate2d, CameraOpticalSource2d};

use super::resolve_camera_optical_candidate_2d;

pub fn collect_camera_optical_candidates_2d(
    sources: &[CameraOpticalSource2d],
) -> Vec<CameraOpticalCandidate2d> {
    sources
        .iter()
        .filter_map(resolve_camera_optical_candidate_2d)
        .collect()
}
