#[derive(Clone, Debug, PartialEq)]
pub struct CameraDepthMotion2d {
    pub camera_z_m: f32,
    pub focus_residual_m: f32,
    pub dolly_signal: f32,
}

impl Default for CameraDepthMotion2d {
    fn default() -> Self {
        Self {
            camera_z_m: 0.0,
            focus_residual_m: 0.0,
            dolly_signal: 0.0,
        }
    }
}

impl CameraDepthMotion2d {
    pub fn normalized(mut self) -> Self {
        self.camera_z_m = finite_or_zero(self.camera_z_m).clamp(-50.0, 50.0);
        self.focus_residual_m = finite_or_zero(self.focus_residual_m).clamp(-5.0, 5.0);
        self.dolly_signal = finite_or_zero(self.dolly_signal).clamp(-1.0, 1.0);
        self
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
