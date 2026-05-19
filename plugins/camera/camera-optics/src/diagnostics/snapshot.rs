use crate::api::CameraOpticalCandidate2d;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CameraOpticalDiagnosticsSnapshot {
    pub candidates: Vec<CameraOpticalCandidate2d>,
}

impl CameraOpticalDiagnosticsSnapshot {
    pub fn format(&self) -> String {
        super::format_camera_optical_candidates_2d(&self.candidates)
    }
}
