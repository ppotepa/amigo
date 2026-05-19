#[derive(Clone, Debug, PartialEq)]
pub struct MotionShutterResponse2dDocument {
    pub enabled: bool,
    pub shutter_angle: f32,
    pub exposure_time_s: f32,
    pub motion_blur: f32,
    pub temporal_accumulation: f32,
}

impl Default for MotionShutterResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            shutter_angle: 180.0,
            exposure_time_s: 1.0 / 60.0,
            motion_blur: 0.0,
            temporal_accumulation: 0.0,
        }
    }
}
