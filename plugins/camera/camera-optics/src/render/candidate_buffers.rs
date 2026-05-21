use crate::api::CameraOpticalCoverage2d;

pub fn coverage_uses_texture_path(coverage: &CameraOpticalCoverage2d) -> bool {
    matches!(coverage, CameraOpticalCoverage2d::LightMapChannel { .. })
}
