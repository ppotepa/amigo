use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxEmbossEdges2d {
    pub mode: PostFxEmbossMode2d,
    pub intensity: f32,
    pub edge_strength: f32,
    pub sample_offset_px: f32,
    pub luma_threshold: f32,
    pub luma_gamma: f32,
    pub specular_radius_px: f32,
    pub distance_falloff: f32,
    pub tint: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFxEmbossMode2d {
    PrebakedImage,
    LightAwareRuntime,
}

impl Default for PostFxEmbossEdges2d {
    fn default() -> Self {
        Self {
            mode: PostFxEmbossMode2d::PrebakedImage,
            intensity: 0.35,
            edge_strength: 1.25,
            sample_offset_px: 1.0,
            luma_threshold: 0.22,
            luma_gamma: 2.2,
            specular_radius_px: 6.0,
            distance_falloff: 0.18,
            tint: [1.0, 1.0, 1.0],
        }
    }
}

impl PostFxEmbossEdges2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            mode: self.mode,
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 2.0),
            edge_strength: finite_or(self.edge_strength, defaults.edge_strength).clamp(0.0, 4.0),
            sample_offset_px: finite_or(self.sample_offset_px, defaults.sample_offset_px)
                .clamp(1.0, 4.0),
            luma_threshold: finite_or(self.luma_threshold, defaults.luma_threshold).clamp(0.0, 1.0),
            luma_gamma: finite_or(self.luma_gamma, defaults.luma_gamma).clamp(0.5, 4.0),
            specular_radius_px: finite_or(self.specular_radius_px, defaults.specular_radius_px)
                .clamp(1.0, 24.0),
            distance_falloff: finite_or(self.distance_falloff, defaults.distance_falloff)
                .clamp(0.01, 2.0),
            tint: [
                finite_or(self.tint[0], defaults.tint[0]).clamp(0.0, 1.0),
                finite_or(self.tint[1], defaults.tint[1]).clamp(0.0, 1.0),
                finite_or(self.tint[2], defaults.tint[2]).clamp(0.0, 1.0),
            ],
        }
    }

    pub fn is_active(&self) -> bool {
        self.intensity > 0.0 && self.edge_strength > 0.0
    }
}
