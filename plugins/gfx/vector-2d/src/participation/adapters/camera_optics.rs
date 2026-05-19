use amigo_camera_optics_plugin::api::CameraOpticalCoverage2d;

use crate::api::Vector2dCoverage;

pub fn vector_coverage_to_camera_optics(coverage: &Vector2dCoverage) -> CameraOpticalCoverage2d {
    match coverage {
        Vector2dCoverage::VectorCoverage {
            entity_name,
            render_layer,
        } => CameraOpticalCoverage2d::VectorCoverage {
            entity_name: entity_name.clone(),
            render_layer: render_layer.clone(),
        },
    }
}
