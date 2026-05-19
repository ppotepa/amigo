#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionShutterResponse2d {
    pub enabled: bool,
    pub shutter_angle: f32,
    pub exposure_time_s: f32,
    pub motion_blur: f32,
    pub temporal_accumulation: f32,
}

impl Default for MotionShutterResponse2d {
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

impl MotionShutterResponse2d {
    pub fn normalized(self) -> Self {
        Self {
            enabled: self.enabled,
            shutter_angle: finite_or_zero(self.shutter_angle).clamp(0.0, 360.0),
            exposure_time_s: finite_or_zero(self.exposure_time_s).clamp(0.0, 10.0),
            motion_blur: finite_or_zero(self.motion_blur).clamp(0.0, 8.0),
            temporal_accumulation: finite_or_zero(self.temporal_accumulation).clamp(0.0, 1.0),
        }
    }

    pub fn is_enabled(self) -> bool {
        let response = self.normalized();
        response.enabled
            && (response.exposure_time_s > 0.0
                || response.motion_blur > 0.0
                || response.temporal_accumulation > 0.0)
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
