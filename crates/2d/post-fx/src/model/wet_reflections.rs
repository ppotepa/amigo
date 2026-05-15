use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WetReflectionsDebugView {
    Final,
    Mask,
    Edges,
    Light,
    Distortion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostFxWetReflections2d {
    pub enabled: bool,
    pub reflection_mask: String,
    pub reflection_mask_invert: bool,
    pub edge_map: Option<String>,
    pub reflection_color: Option<String>,
    pub noise_normal: Option<String>,
    pub blur_px: f32,
    pub distortion_px: f32,
    pub shimmer_strength: f32,
    pub ripple_strength: f32,
    pub wet_darken: f32,
    pub specular_boost: f32,
    pub edge_power: f32,
    pub light_reflection_strength: f32,
    pub foreground_strength: f32,
    pub background_strength: f32,
    pub horizon_y: f32,
    pub noise_scale: f32,
    pub noise_speed: f32,
    pub ripple_speed: f32,
    pub debug_view: WetReflectionsDebugView,
}

impl Default for PostFxWetReflections2d {
    fn default() -> Self {
        Self {
            enabled: true,
            reflection_mask: String::new(),
            reflection_mask_invert: true,
            edge_map: None,
            reflection_color: None,
            noise_normal: None,
            blur_px: 1.5,
            distortion_px: 0.8,
            shimmer_strength: 0.04,
            ripple_strength: 0.02,
            wet_darken: 0.06,
            specular_boost: 0.25,
            edge_power: 1.35,
            light_reflection_strength: 0.65,
            foreground_strength: 1.0,
            background_strength: 0.12,
            horizon_y: 0.42,
            noise_scale: 2.5,
            noise_speed: 0.035,
            ripple_speed: 0.08,
            debug_view: WetReflectionsDebugView::Final,
        }
    }
}

impl PostFxWetReflections2d {
    pub fn normalized(mut self) -> Self {
        self.blur_px = self.blur_px.clamp(0.0, 12.0);
        self.distortion_px = self.distortion_px.clamp(0.0, 16.0);
        self.shimmer_strength = self.shimmer_strength.clamp(0.0, 1.0);
        self.ripple_strength = self.ripple_strength.clamp(0.0, 1.0);
        self.wet_darken = self.wet_darken.clamp(0.0, 1.0);
        self.specular_boost = self.specular_boost.clamp(0.0, 4.0);
        self.edge_power = self.edge_power.clamp(0.25, 8.0);
        self.light_reflection_strength = self.light_reflection_strength.clamp(0.0, 4.0);
        self.foreground_strength = self.foreground_strength.clamp(0.0, 4.0);
        self.background_strength = self.background_strength.clamp(0.0, 4.0);
        self.horizon_y = self.horizon_y.clamp(0.0, 1.0);
        self.noise_scale = self.noise_scale.clamp(0.01, 64.0);
        self.noise_speed = self.noise_speed.clamp(-8.0, 8.0);
        self.ripple_speed = self.ripple_speed.clamp(-8.0, 8.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.enabled && !self.reflection_mask.trim().is_empty()
    }
}
