use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRamp2d {
    pub palette_size: u32,
    pub dither_strength: f32,
    pub dither_scale: f32,
    pub layered_dither: f32,
    pub opacity: f32,
    pub luma_preserve: f32,
    pub highlight_bias: f32,
    pub shadow_bias: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub seed: u32,
}

impl Default for ColorRamp2d {
    fn default() -> Self {
        Self {
            palette_size: 32,
            dither_strength: 0.42,
            dither_scale: 1.0,
            layered_dither: 0.35,
            opacity: 1.0,
            luma_preserve: 0.55,
            highlight_bias: 0.25,
            shadow_bias: 0.65,
            contrast: 1.18,
            saturation: 0.85,
            gamma: 2.15,
            seed: 1931,
        }
    }
}

impl ColorRamp2d {
    pub fn normalized(mut self) -> Self {
        self.palette_size = self.palette_size.clamp(2, 256);
        self.dither_strength = finite_or(self.dither_strength, 0.42).clamp(0.0, 1.0);
        self.dither_scale = finite_or(self.dither_scale, 1.0).clamp(1.0, 8.0);
        self.layered_dither = finite_or(self.layered_dither, 0.35).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self.luma_preserve = finite_or(self.luma_preserve, 0.55).clamp(0.0, 1.0);
        self.highlight_bias = finite_or(self.highlight_bias, 0.25).clamp(0.0, 1.0);
        self.shadow_bias = finite_or(self.shadow_bias, 0.65).clamp(0.0, 1.0);
        self.contrast = finite_or(self.contrast, 1.18).clamp(0.25, 2.0);
        self.saturation = finite_or(self.saturation, 0.85).clamp(0.0, 2.0);
        self.gamma = finite_or(self.gamma, 2.15).clamp(1.0, 3.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.palette_size >= 2 && self.opacity > 0.0
    }
}

impl From<ColorQuantize2d> for ColorRamp2d {
    fn from(effect: ColorQuantize2d) -> Self {
        let effect = effect.normalized();
        Self {
            palette_size: effect.palette_size,
            dither_strength: effect.dither_strength,
            dither_scale: effect.dither_scale,
            layered_dither: effect.layered_dither,
            opacity: effect.opacity,
            luma_preserve: effect.luma_preserve,
            highlight_bias: effect.highlight_bias,
            shadow_bias: effect.shadow_bias,
            contrast: effect.contrast,
            saturation: effect.saturation,
            gamma: effect.gamma,
            seed: effect.seed,
        }
    }
}
