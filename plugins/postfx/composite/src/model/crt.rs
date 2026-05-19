use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crt2d {
    pub scanline_opacity: f32,
    pub scanline_frequency_px: f32,
    pub rgb_split_px: f32,
    pub curvature: f32,
    pub vignette: f32,
    pub phosphor_mask: f32,
    pub brightness_compensation: f32,
}

impl Default for Crt2d {
    fn default() -> Self {
        Self {
            scanline_opacity: 0.12,
            scanline_frequency_px: 1.5,
            rgb_split_px: 1.0,
            curvature: 0.03,
            vignette: 0.22,
            phosphor_mask: 0.04,
            brightness_compensation: 1.05,
        }
    }
}

impl Crt2d {
    pub fn normalized(mut self) -> Self {
        self.scanline_opacity = finite_or(self.scanline_opacity, 0.12).clamp(0.0, 1.0);
        self.scanline_frequency_px = finite_or(self.scanline_frequency_px, 1.5).clamp(0.5, 8.0);
        self.rgb_split_px = finite_or(self.rgb_split_px, 1.0).clamp(0.0, 8.0);
        self.curvature = finite_or(self.curvature, 0.03).clamp(0.0, 0.5);
        self.vignette = finite_or(self.vignette, 0.22).clamp(0.0, 1.0);
        self.phosphor_mask = finite_or(self.phosphor_mask, 0.04).clamp(0.0, 1.0);
        self.brightness_compensation =
            finite_or(self.brightness_compensation, 1.05).clamp(0.0, 4.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.scanline_opacity > 0.0
            || self.rgb_split_px > 0.0
            || self.curvature > 0.0
            || self.vignette > 0.0
            || self.phosphor_mask > 0.0
    }
}
