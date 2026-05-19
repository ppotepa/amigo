use amigo_shutter_motion_plugin::api::{
    MotionShutterCoverage2d, MotionShutterResponse2d, MotionShutterSource2d,
};

use crate::api::ParticleEmitter2dSource;

pub fn particle_emitter_to_shutter_motion(source: &ParticleEmitter2dSource) -> MotionShutterSource2d {
    MotionShutterSource2d {
        owner: source.id.clone(),
        declared: source.motion_response > 0.0,
        coverage: MotionShutterCoverage2d::RenderLayer {
            layer_id: source.render_layer.clone(),
        },
        response: MotionShutterResponse2d {
            enabled: source.motion_response > 0.0,
            motion_blur: source.motion_response,
            ..MotionShutterResponse2d::default()
        },
    }
}
