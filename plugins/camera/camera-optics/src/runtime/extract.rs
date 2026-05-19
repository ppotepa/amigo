use crate::api::{CameraOpticalCandidate2d, CameraOpticalSource2d};

use super::collect_camera_optical_candidates_2d;

pub fn extract_camera_optical_candidates_2d(
    sources: &[CameraOpticalSource2d],
) -> Vec<CameraOpticalCandidate2d> {
    collect_camera_optical_candidates_2d(sources)
}
