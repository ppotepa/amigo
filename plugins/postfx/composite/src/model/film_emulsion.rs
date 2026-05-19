use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmEmulsion2d {
    pub color_shift: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub toe: f32,
    pub shoulder: f32,
    pub black_lift: f32,
    pub push_pull: f32,
    pub opacity: f32,
}

impl Default for FilmEmulsion2d {
    fn default() -> Self {
        Self {
            color_shift: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            toe: 0.45,
            shoulder: 0.65,
            black_lift: 0.02,
            push_pull: 0.0,
            opacity: 1.0,
        }
    }
}

impl FilmEmulsion2d {
    pub fn normalized(mut self) -> Self {
        self.color_shift = finite_or(self.color_shift, 0.0).clamp(-1.0, 1.0);
        self.contrast = finite_or(self.contrast, 1.0).clamp(0.25, 4.0);
        self.saturation = finite_or(self.saturation, 1.0).clamp(0.0, 4.0);
        self.toe = finite_or(self.toe, 0.45).clamp(0.0, 1.0);
        self.shoulder = finite_or(self.shoulder, 0.65).clamp(0.0, 1.0);
        self.black_lift = finite_or(self.black_lift, 0.02).clamp(0.0, 0.4);
        self.push_pull = finite_or(self.push_pull, 0.0).clamp(-4.0, 4.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0
    }
}
