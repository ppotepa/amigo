use amigo_plugin_api::TargetId;

use crate::api::CameraOpticalCandidate2d;

pub fn optical_candidate_color_rgba_for_target(
    candidate: &CameraOpticalCandidate2d,
    target: &TargetId,
) -> [f32; 4] {
    let gain = if candidate
        .target_ids
        .iter()
        .any(|candidate_target| candidate_target == target)
    {
        if target.0 == "SceneHighlight" {
            candidate.highlight_gain()
        } else if target.0 == "SceneEmissive" {
            candidate.emissive_gain()
        } else {
            0.0
        }
    } else {
        0.0
    };

    [
        candidate.color_rgba[0] * gain,
        candidate.color_rgba[1] * gain,
        candidate.color_rgba[2] * gain,
        candidate.color_rgba[3],
    ]
}
