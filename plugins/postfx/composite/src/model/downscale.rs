use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Downscale2d {
    pub factor: f32,
    pub opacity: f32,
}

impl Default for Downscale2d {
    fn default() -> Self {
        Self {
            factor: 2.0,
            opacity: 1.0,
        }
    }
}

impl Downscale2d {
    pub fn normalized(mut self) -> Self {
        self.factor = finite_or(self.factor, 2.0).clamp(1.0, 16.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.factor > 1.0 && self.opacity > 0.0
    }
}
