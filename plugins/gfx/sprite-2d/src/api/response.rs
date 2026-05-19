#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite2dRenderResponse {
    pub visible: bool,
    pub opacity: f32,
}

impl Default for Sprite2dRenderResponse {
    fn default() -> Self {
        Self {
            visible: true,
            opacity: 1.0,
        }
    }
}

impl Sprite2dRenderResponse {
    pub fn normalized(self) -> Self {
        Self {
            visible: self.visible,
            opacity: if self.opacity.is_finite() {
                self.opacity.clamp(0.0, 1.0)
            } else {
                0.0
            },
        }
    }
}
