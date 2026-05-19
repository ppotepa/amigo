use amigo_camera_optics_plugin::api::CameraOpticalResponse2d;

use crate::api::Material2dSource;

pub fn material_to_camera_optics_response(material: &Material2dSource) -> CameraOpticalResponse2d {
    CameraOpticalResponse2d {
        enabled: material.opacity > 0.0,
        intensity: material.opacity,
        bloom: 0.0,
        glare: 0.0,
        ghosting: 0.0,
        streaks: 0.0,
        chromatic_smear: 0.0,
        dirt_response: 0.0,
        halation: 0.0,
        threshold: 0.0,
    }
}
