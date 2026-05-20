use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WetReflections2dDocument {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub masks: WetReflectionsMasksDocument,
    #[serde(default)]
    pub surface: WetReflectionsSurfaceDocument,
    #[serde(default)]
    pub perspective: WetReflectionsPerspectiveDocument,
    #[serde(default)]
    pub animation: WetReflectionsAnimationDocument,
    #[serde(default)]
    pub light_response: WetReflectionsLightResponseDocument,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WetReflectionsMasksDocument {
    #[serde(default)]
    pub reflection: Option<String>,
    #[serde(default)]
    pub reflection_invert: Option<bool>,
    #[serde(default)]
    pub edges: Option<String>,
    #[serde(default)]
    pub reflection_color: Option<String>,
    #[serde(default)]
    pub noise_normal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WetReflectionsSurfaceDocument {
    #[serde(default = "default_wet_blur_px")]
    pub blur_px: f32,
    #[serde(default = "default_wet_distortion_px")]
    pub distortion_px: f32,
    #[serde(default = "default_wet_shimmer_strength")]
    pub shimmer_strength: f32,
    #[serde(default = "default_wet_ripple_strength")]
    pub ripple_strength: f32,
    #[serde(default = "default_wet_darken")]
    pub wet_darken: f32,
    #[serde(default = "default_wet_specular_boost")]
    pub specular_boost: f32,
    #[serde(default = "default_wet_edge_power")]
    pub edge_power: f32,
    #[serde(default = "default_wet_light_reflection_strength")]
    pub light_reflection_strength: f32,
}

impl Default for WetReflectionsSurfaceDocument {
    fn default() -> Self {
        Self {
            blur_px: default_wet_blur_px(),
            distortion_px: default_wet_distortion_px(),
            shimmer_strength: default_wet_shimmer_strength(),
            ripple_strength: default_wet_ripple_strength(),
            wet_darken: default_wet_darken(),
            specular_boost: default_wet_specular_boost(),
            edge_power: default_wet_edge_power(),
            light_reflection_strength: default_wet_light_reflection_strength(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WetReflectionsPerspectiveDocument {
    #[serde(default = "default_wet_foreground_strength")]
    pub foreground_strength: f32,
    #[serde(default = "default_wet_background_strength")]
    pub background_strength: f32,
    #[serde(default = "default_wet_horizon_y")]
    pub horizon_y: f32,
}

impl Default for WetReflectionsPerspectiveDocument {
    fn default() -> Self {
        Self {
            foreground_strength: default_wet_foreground_strength(),
            background_strength: default_wet_background_strength(),
            horizon_y: default_wet_horizon_y(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WetReflectionsAnimationDocument {
    #[serde(default = "default_wet_noise_scale")]
    pub noise_scale: f32,
    #[serde(default = "default_wet_noise_speed")]
    pub noise_speed: f32,
    #[serde(default = "default_wet_ripple_speed")]
    pub ripple_speed: f32,
}

impl Default for WetReflectionsAnimationDocument {
    fn default() -> Self {
        Self {
            noise_scale: default_wet_noise_scale(),
            noise_speed: default_wet_noise_speed(),
            ripple_speed: default_wet_ripple_speed(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WetReflectionsLightResponseDocument {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub strength: Option<f32>,
    #[serde(default)]
    pub edge_power: Option<f32>,
}
