#[derive(Clone, Debug, PartialEq)]
pub struct FocusDepthResponse2dDocument {
    pub enabled: bool,
    pub strength: f32,
    pub focus_width_m: f32,
    pub max_blur_px: f32,
}

impl Default for FocusDepthResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: 0.0,
            focus_width_m: 1.0,
            max_blur_px: 0.0,
        }
    }
}
