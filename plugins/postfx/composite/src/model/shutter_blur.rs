use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShutterBlur2d {
    pub exposure_seconds: f32,
    pub fps: f32,
    pub shutter_angle: f32,
    pub opacity: f32,
    pub history_mix: f32,
    pub history_mix_2: f32,
    pub edge_rejection: f32,
    pub luma_threshold: f32,
    pub frame_hold: bool,
}

impl Default for ShutterBlur2d {
    fn default() -> Self {
        Self {
            exposure_seconds: 1.0 / 48.0,
            fps: 24.0,
            shutter_angle: 180.0,
            opacity: 0.72,
            history_mix: 0.0,
            history_mix_2: 0.0,
            edge_rejection: 0.35,
            luma_threshold: 0.04,
            frame_hold: false,
        }
    }
}

impl ShutterBlur2d {
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.exposure_seconds =
            finite_or(self.exposure_seconds, defaults.exposure_seconds).clamp(1.0 / 8000.0, 2.0);
        self.fps = finite_or(self.fps, defaults.fps).clamp(1.0, 240.0);
        self.shutter_angle =
            finite_or(self.shutter_angle, defaults.shutter_angle).clamp(0.0, 360.0);
        self.opacity = finite_or(self.opacity, defaults.opacity).clamp(0.0, 1.0);
        self.history_mix = finite_or(self.history_mix, defaults.history_mix).clamp(0.0, 1.0);
        self.history_mix_2 = finite_or(self.history_mix_2, defaults.history_mix_2).clamp(0.0, 1.0);
        self.edge_rejection =
            finite_or(self.edge_rejection, defaults.edge_rejection).clamp(0.0, 1.0);
        self.luma_threshold =
            finite_or(self.luma_threshold, defaults.luma_threshold).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0 && self.exposure_seconds > 0.0
    }

    pub fn exposure_frames(&self, dt: f32) -> f32 {
        self.exposure_seconds / dt.max(1.0 / 240.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutter_speed_seconds_reports_exposure_frames() {
        let effect = ShutterBlur2d {
            exposure_seconds: 0.1,
            ..ShutterBlur2d::default()
        };

        assert!((effect.exposure_frames(1.0 / 60.0) - 6.0).abs() < 0.01);
    }
}
