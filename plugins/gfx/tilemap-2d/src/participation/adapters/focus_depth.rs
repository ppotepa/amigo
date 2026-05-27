use amigo_plugin_api::TargetId;

use crate::api::Tilemap2dCandidate;

pub fn tilemap_to_focus_depth(candidate: &Tilemap2dCandidate) -> TargetId {
    TargetId(format!(
        "focus-depth.render-layer.{}",
        candidate.render_layer
    ))
}
