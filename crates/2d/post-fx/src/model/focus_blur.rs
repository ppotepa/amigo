use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusTarget2d {
    None,
    RenderLayer { layer: String },
    SceneObject { object: String },
    Depth { value: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusBlurDebugView2d {
    Final,
    Depth,
    CircleOfConfusion,
    FocusBand,
    Split,
    HighlightMask,
}

impl FocusBlurDebugView2d {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "depth" | "depth_map" => Self::Depth,
            "coc" | "circle_of_confusion" | "signed_coc" | "near_far_coc" => {
                Self::CircleOfConfusion
            }
            "focus" | "focus_band" => Self::FocusBand,
            "split" => Self::Split,
            "highlight" | "highlight_mask" | "bokeh_highlight" => Self::HighlightMask,
            _ => Self::Final,
        }
    }

    pub fn shader_value(self) -> f32 {
        match self {
            Self::Final => 0.0,
            Self::Depth => 1.0,
            Self::CircleOfConfusion => 2.0,
            Self::FocusBand => 3.0,
            Self::Split => 4.0,
            Self::HighlightMask => 5.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FocusBlur2d {
    pub focus: FocusTarget2d,
    pub f_stop: f32,
    pub focus_distance_m: f32,
    pub focus_radius: f32,
    pub blur_radius: f32,
    pub anamorphic_ratio: f32,
    pub cat_eye_bokeh: f32,
    pub focus_breathing: f32,
    pub opacity: f32,
    pub depth_map: Option<String>,
    pub affected_layers: Vec<String>,
    pub focal_length_mm: f32,
    pub max_blur_px: f32,
    pub depth_contrast: f32,
    pub focus_width: f32,
    pub foreground_blur_boost: f32,
    pub background_blur_boost: f32,
    pub edge_aware: bool,
    pub invert_depth: bool,
    pub debug_view: FocusBlurDebugView2d,
    pub aperture_blades: u32,
    pub aperture_roundness: f32,
    pub aperture_rotation_degrees: f32,
    pub sample_count: u32,
    pub highlight_threshold: f32,
    pub highlight_knee: f32,
    pub highlight_gain: f32,
    pub highlight_saturation: f32,
}

impl Default for FocusBlur2d {
    fn default() -> Self {
        Self {
            focus: FocusTarget2d::None,
            f_stop: 8.0,
            focus_distance_m: 5.0,
            focus_radius: 0.18,
            blur_radius: 8.0,
            anamorphic_ratio: 1.35,
            cat_eye_bokeh: 0.0,
            focus_breathing: 0.0,
            opacity: 1.0,
            depth_map: None,
            affected_layers: Vec::new(),
            focal_length_mm: 50.0,
            max_blur_px: 28.0,
            depth_contrast: 1.0,
            focus_width: 0.055,
            foreground_blur_boost: 1.15,
            background_blur_boost: 1.0,
            edge_aware: true,
            invert_depth: false,
            debug_view: FocusBlurDebugView2d::Final,
            aperture_blades: 7,
            aperture_roundness: 0.72,
            aperture_rotation_degrees: 0.0,
            sample_count: 64,
            highlight_threshold: 0.68,
            highlight_knee: 0.18,
            highlight_gain: 1.45,
            highlight_saturation: 1.10,
        }
    }
}

impl FocusBlur2d {
    pub fn normalized(mut self) -> Self {
        self.f_stop = finite_or(self.f_stop, 8.0).clamp(0.7, 32.0);
        self.focus_distance_m = finite_or(self.focus_distance_m, 5.0).clamp(0.2, 1000.0);
        self.focus_radius = finite_or(self.focus_radius, 0.18).clamp(0.01, 1.0);
        self.blur_radius = finite_or(self.blur_radius, 8.0).clamp(0.0, 32.0);
        self.anamorphic_ratio = finite_or(self.anamorphic_ratio, 1.35).clamp(0.2, 4.0);
        self.cat_eye_bokeh = finite_or(self.cat_eye_bokeh, 0.0).clamp(0.0, 1.0);
        self.focus_breathing = finite_or(self.focus_breathing, 0.0).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 1.0).clamp(0.0, 1.0);
        self.focal_length_mm = finite_or(self.focal_length_mm, 50.0).clamp(18.0, 135.0);
        self.max_blur_px = finite_or(self.max_blur_px, 28.0).clamp(0.0, 90.0);
        self.depth_contrast = finite_or(self.depth_contrast, 1.0).clamp(0.4, 2.4);
        self.focus_width = finite_or(self.focus_width, 0.055).clamp(0.005, 0.22);
        self.foreground_blur_boost = finite_or(self.foreground_blur_boost, 1.15).clamp(0.25, 2.5);
        self.background_blur_boost = finite_or(self.background_blur_boost, 1.0).clamp(0.25, 2.5);
        self.affected_layers = normalized_layer_list(self.affected_layers);
        self.aperture_blades = self.aperture_blades.clamp(0, 12);
        if self.aperture_blades > 0 && self.aperture_blades < 3 {
            self.aperture_blades = 3;
        }
        self.aperture_roundness = finite_or(self.aperture_roundness, 0.72).clamp(0.0, 1.0);
        self.aperture_rotation_degrees =
            finite_or(self.aperture_rotation_degrees, 0.0).rem_euclid(360.0);
        self.sample_count = self.sample_count.clamp(12, 96);
        self.highlight_threshold = finite_or(self.highlight_threshold, 0.68).clamp(0.0, 4.0);
        self.highlight_knee = finite_or(self.highlight_knee, 0.18).clamp(0.001, 2.0);
        self.highlight_gain = finite_or(self.highlight_gain, 1.45).clamp(0.0, 8.0);
        self.highlight_saturation = finite_or(self.highlight_saturation, 1.10).clamp(0.0, 3.0);
        if let FocusTarget2d::Depth { value } = &mut self.focus {
            *value = value.clamp(0.0, 1.0);
        }
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0
            && (self.blur_radius > 0.0 || self.max_blur_px > 0.0)
            && !matches!(self.focus, FocusTarget2d::None)
    }
}

fn normalized_layer_list(layers: Vec<String>) -> Vec<String> {
    let mut layers = layers
        .into_iter()
        .map(|layer| layer.trim().to_owned())
        .filter(|layer| !layer.is_empty())
        .collect::<Vec<_>>();
    layers.sort();
    layers.dedup();
    layers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_blur_bokeh_params_are_normalized() {
        let effect = FocusBlur2d {
            affected_layers: vec![
                " weather.rain ".to_owned(),
                String::new(),
                "background.city".to_owned(),
                "background.city".to_owned(),
            ],
            aperture_blades: 2,
            aperture_roundness: 9.0,
            aperture_rotation_degrees: -45.0,
            sample_count: 999,
            highlight_threshold: f32::NAN,
            highlight_knee: -1.0,
            highlight_gain: 99.0,
            highlight_saturation: 99.0,
            ..FocusBlur2d::default()
        }
        .normalized();

        assert_eq!(effect.aperture_blades, 3);
        assert_eq!(
            effect.affected_layers,
            vec!["background.city".to_owned(), "weather.rain".to_owned()]
        );
        assert_eq!(effect.sample_count, 96);
        assert!((0.0..=1.0).contains(&effect.aperture_roundness));
        assert!((0.0..360.0).contains(&effect.aperture_rotation_degrees));
        assert!(effect.highlight_threshold.is_finite());
        assert!(effect.highlight_knee > 0.0);
        assert!(effect.highlight_gain <= 8.0);
        assert!(effect.highlight_saturation <= 3.0);
    }
}
