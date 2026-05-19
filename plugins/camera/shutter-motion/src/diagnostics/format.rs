use crate::api::{MotionShutterCandidate2d, MotionShutterCoverage2d};

pub fn format_motion_shutter_candidates_2d(candidates: &[MotionShutterCandidate2d]) -> String {
    if candidates.is_empty() {
        return "motion_shutter.candidates: none".to_owned();
    }

    candidates
        .iter()
        .map(|candidate| {
            format!(
                "owner={} coverage={} status={:?} reason={} targets={}",
                candidate.owner,
                coverage_label(&candidate.coverage),
                candidate.status,
                candidate.reason,
                candidate
                    .target_ids
                    .iter()
                    .map(|target| target.0.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn coverage_label(coverage: &MotionShutterCoverage2d) -> &'static str {
    match coverage {
        MotionShutterCoverage2d::SceneVelocity => "scene_velocity",
        MotionShutterCoverage2d::CameraMotion => "camera_motion",
        MotionShutterCoverage2d::RenderLayer { .. } => "render_layer",
        MotionShutterCoverage2d::Unsupported { .. } => "unsupported",
    }
}
