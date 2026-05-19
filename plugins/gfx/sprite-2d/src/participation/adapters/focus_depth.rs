use amigo_plugin_api::TargetId;

use crate::api::Sprite2dCoverage;

pub fn sprite_coverage_to_focus_depth(coverage: &Sprite2dCoverage) -> Option<TargetId> {
    match coverage {
        Sprite2dCoverage::TextureAlpha { render_layer, .. } => {
            Some(TargetId(format!("focus-depth.render-layer.{render_layer}")))
        }
        Sprite2dCoverage::Unsupported { .. } => None,
    }
}
