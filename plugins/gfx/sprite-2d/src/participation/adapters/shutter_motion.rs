use amigo_shutter_motion_plugin::api::MotionShutterCoverage2d;

use crate::api::Sprite2dCoverage;

pub fn sprite_coverage_to_shutter_motion(
    coverage: &Sprite2dCoverage,
) -> Option<MotionShutterCoverage2d> {
    match coverage {
        Sprite2dCoverage::TextureAlpha { render_layer, .. } => {
            Some(MotionShutterCoverage2d::RenderLayer {
                layer_id: render_layer.clone(),
            })
        }
        Sprite2dCoverage::Unsupported { .. } => None,
    }
}
