use amigo_plugin_api::{StandardTarget, TargetId};

use crate::api::CameraOpticalCandidate2d;

pub fn targets_scene_highlight_buffer(
    has_visual_map: bool,
    candidates: &[CameraOpticalCandidate2d],
) -> bool {
    has_visual_map
        || candidates
            .iter()
            .any(|candidate| candidate.targets_scene_highlight())
}

pub fn targets_scene_emissive_buffer(
    has_visual_map: bool,
    candidates: &[CameraOpticalCandidate2d],
) -> bool {
    has_visual_map
        || candidates
            .iter()
            .any(|candidate| candidate.targets_scene_emissive())
}

pub fn scene_highlight_target_id() -> TargetId {
    StandardTarget::SceneHighlight.id()
}

pub fn scene_emissive_target_id() -> TargetId {
    StandardTarget::SceneEmissive.id()
}
