use amigo_camera_core_plugin::api::CameraDepthMotion2d;

pub fn effective_distance_after_camera_z_m(distance_m: f32, camera_z_m: f32) -> f32 {
    if !distance_m.is_finite() {
        return 0.05;
    }
    (distance_m - finite_or_zero(camera_z_m)).max(0.05)
}

pub fn effective_layer_distance_m(
    distance_m: Option<f32>,
    motion: &CameraDepthMotion2d,
) -> Option<f32> {
    distance_m.map(|distance| effective_distance_after_camera_z_m(distance, motion.camera_z_m))
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
