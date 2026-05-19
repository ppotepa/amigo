use amigo_focus_depth_plugin::api::FocusDepthCoverage2d;

use crate::api::Tilemap2dCandidate;

pub fn tilemap_to_focus_depth(candidate: &Tilemap2dCandidate) -> FocusDepthCoverage2d {
    FocusDepthCoverage2d::RenderLayer {
        layer_id: candidate.render_layer.clone(),
    }
}
