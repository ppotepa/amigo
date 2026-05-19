use amigo_shutter_motion_plugin::api::{
    MotionShutterCoverage2d, MotionShutterResponse2d, MotionShutterSource2d,
};

use crate::api::Trail2dSource;

pub fn trail_to_shutter_motion(source: &Trail2dSource) -> MotionShutterSource2d {
    MotionShutterSource2d {
        owner: source.id.clone(),
        declared: source.length_px > 0.0,
        coverage: MotionShutterCoverage2d::RenderLayer {
            layer_id: source.render_layer.clone(),
        },
        response: MotionShutterResponse2d {
            enabled: source.length_px > 0.0,
            motion_blur: (source.length_px / 100.0).clamp(0.0, 8.0),
            ..MotionShutterResponse2d::default()
        },
    }
}
