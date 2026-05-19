#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusDepthResponse2d {
    pub enabled: bool,
    pub strength: f32,
    pub focus_width_m: f32,
    pub max_blur_px: f32,
}

impl Default for FocusDepthResponse2d {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: 0.0,
            focus_width_m: 1.0,
            max_blur_px: 0.0,
        }
    }
}

impl FocusDepthResponse2d {
    pub fn normalized(self) -> Self {
        Self {
            enabled: self.enabled,
            strength: finite_or_zero(self.strength).clamp(0.0, 8.0),
            focus_width_m: finite_or_zero(self.focus_width_m).max(0.01),
            max_blur_px: finite_or_zero(self.max_blur_px).clamp(0.0, 128.0),
        }
    }

    pub fn is_enabled(self) -> bool {
        let response = self.normalized();
        response.enabled && (response.strength > 0.0 || response.max_blur_px > 0.0)
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
