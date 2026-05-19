use crate::api::MotionShutterResponse2d;

use super::MotionShutterResponse2dDocument;

pub fn motion_shutter_response_from_document(
    document: MotionShutterResponse2dDocument,
) -> MotionShutterResponse2d {
    MotionShutterResponse2d {
        enabled: document.enabled,
        shutter_angle: document.shutter_angle,
        exposure_time_s: document.exposure_time_s,
        motion_blur: document.motion_blur,
        temporal_accumulation: document.temporal_accumulation,
    }
    .normalized()
}
