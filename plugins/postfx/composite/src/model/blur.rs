use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxBlur2d {
    pub radius: f32,
    pub downsample: f32,
    pub intensity: f32,
}

impl Default for PostFxBlur2d {
    fn default() -> Self {
        Self {
            radius: 12.0,
            downsample: 0.5,
            intensity: 1.0,
        }
    }
}

impl PostFxBlur2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            radius: finite_or(self.radius, defaults.radius).clamp(0.0, 128.0),
            downsample: finite_or(self.downsample, defaults.downsample).clamp(0.125, 1.0),
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 4.0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.radius > 0.0 && self.intensity > 0.0
    }
}
