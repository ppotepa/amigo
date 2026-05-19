use crate::api::{FocusDepthCandidate2d, FocusDepthCoverage2d};

pub fn format_focus_depth_candidates_2d(candidates: &[FocusDepthCandidate2d]) -> String {
    if candidates.is_empty() {
        return "focus_depth.candidates: none".to_owned();
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

fn coverage_label(coverage: &FocusDepthCoverage2d) -> &'static str {
    match coverage {
        FocusDepthCoverage2d::SceneDepth => "scene_depth",
        FocusDepthCoverage2d::RenderLayer { .. } => "render_layer",
        FocusDepthCoverage2d::SceneObject { .. } => "scene_object",
        FocusDepthCoverage2d::Distance { .. } => "distance",
        FocusDepthCoverage2d::Unsupported { .. } => "unsupported",
    }
}
