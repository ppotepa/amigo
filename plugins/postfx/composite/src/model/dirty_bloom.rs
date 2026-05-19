use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirtyBloom2d {
    pub threshold: f32,
    pub strength: f32,
    pub small_radius_px: f32,
    pub medium_radius_px: f32,
    pub large_radius_px: f32,
    pub dirty_noise: f32,
    pub halation_strength: f32,
    pub reflection_smear_x_px: f32,
    pub reflection_smear_y_px: f32,
    pub seed: u32,
}

impl Default for DirtyBloom2d {
    fn default() -> Self {
        Self {
            threshold: 0.62,
            strength: 0.75,
            small_radius_px: 3.0,
            medium_radius_px: 12.0,
            large_radius_px: 32.0,
            dirty_noise: 0.18,
            halation_strength: 0.22,
            reflection_smear_x_px: 6.0,
            reflection_smear_y_px: 28.0,
            seed: 4242,
        }
    }
}

impl DirtyBloom2d {
    pub fn normalized(mut self) -> Self {
        self.threshold = finite_or(self.threshold, 0.62).clamp(0.0, 2.0);
        self.strength = finite_or(self.strength, 0.75).clamp(0.0, 4.0);
        self.small_radius_px = finite_or(self.small_radius_px, 3.0).clamp(0.0, 64.0);
        self.medium_radius_px = finite_or(self.medium_radius_px, 12.0).clamp(0.0, 128.0);
        self.large_radius_px = finite_or(self.large_radius_px, 32.0).clamp(0.0, 256.0);
        self.dirty_noise = finite_or(self.dirty_noise, 0.18).clamp(0.0, 1.0);
        self.halation_strength = finite_or(self.halation_strength, 0.22).clamp(0.0, 2.0);
        self.reflection_smear_x_px = finite_or(self.reflection_smear_x_px, 6.0).clamp(0.0, 128.0);
        self.reflection_smear_y_px = finite_or(self.reflection_smear_y_px, 28.0).clamp(0.0, 256.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.strength > 0.0
    }
}
