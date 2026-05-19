use amigo_focus_depth_plugin::api::FocusDepthCoverage2d;

use crate::api::Sprite2dCoverage;

pub fn sprite_coverage_to_focus_depth(coverage: &Sprite2dCoverage) -> Option<FocusDepthCoverage2d> {
    match coverage {
        Sprite2dCoverage::TextureAlpha { render_layer, .. } => {
            Some(FocusDepthCoverage2d::RenderLayer {
                layer_id: render_layer.clone(),
            })
        }
        Sprite2dCoverage::Unsupported { .. } => None,
    }
}
