use amigo_camera_optics_plugin::api::CameraOpticalCoverage2d;

use crate::api::Sprite2dCoverage;

pub fn sprite_coverage_to_camera_optics(
    coverage: &Sprite2dCoverage,
) -> Option<CameraOpticalCoverage2d> {
    match coverage {
        Sprite2dCoverage::TextureAlpha {
            entity_name,
            render_layer,
        } => Some(CameraOpticalCoverage2d::TextureAlpha {
            entity_name: entity_name.clone(),
            render_layer: render_layer.clone(),
        }),
        Sprite2dCoverage::Unsupported { .. } => None,
    }
}
