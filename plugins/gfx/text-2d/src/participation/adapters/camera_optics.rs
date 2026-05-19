use amigo_camera_optics_plugin::api::CameraOpticalCoverage2d;

use crate::api::Text2dCoverage;

pub fn text_coverage_to_camera_optics(coverage: &Text2dCoverage) -> CameraOpticalCoverage2d {
    match coverage {
        Text2dCoverage::Glyphs {
            entity_name,
            render_layer,
        } => CameraOpticalCoverage2d::Glyphs {
            entity_name: entity_name.clone(),
            render_layer: render_layer.clone(),
        },
    }
}
