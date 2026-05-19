use amigo_shutter_motion_plugin::api::{MotionShutterCoverage2d, MotionShutterResponse2d, MotionShutterSource2d};

use crate::api::BeaconLight2dSource;

pub fn animated_beacon_to_shutter_motion(beacon: &BeaconLight2dSource) -> Option<MotionShutterSource2d> {
    beacon.animated.then(|| MotionShutterSource2d {
        owner: beacon.id.clone(),
        declared: true,
        coverage: MotionShutterCoverage2d::CameraMotion,
        response: MotionShutterResponse2d {
            enabled: true,
            motion_blur: beacon.intensity,
            ..MotionShutterResponse2d::default()
        },
    })
}
