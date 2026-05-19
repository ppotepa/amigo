use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PostFx2dDocument {
    ColorQuantize(ColorQuantize2dDocument),
    ColorRamp(ColorRamp2dDocument),
    Crt(Crt2dDocument),
    Downscale(Downscale2dDocument),
    DirtyBloom(DirtyBloom2dDocument),
    FilmNoise(FilmNoise2dDocument),
    LensDroplets(LensDroplets2dDocument),
    RainGlass(RainGlass2dDocument),
    ShutterBlur(ShutterBlur2dDocument),
    WetReflections(WetReflections2dDocument),
}

impl PostFx2dDocument {
    pub fn id(&self) -> &str {
        match self {
            Self::ColorQuantize(effect) => effect.id.as_str(),
            Self::ColorRamp(effect) => effect.id.as_str(),
            Self::Crt(effect) => effect.id.as_str(),
            Self::Downscale(effect) => effect.id.as_str(),
            Self::DirtyBloom(effect) => effect.id.as_str(),
            Self::FilmNoise(effect) => effect.id.as_str(),
            Self::LensDroplets(effect) => effect.id.as_str(),
            Self::RainGlass(effect) => effect.id.as_str(),
            Self::ShutterBlur(effect) => effect.id.as_str(),
            Self::WetReflections(effect) => effect.id.as_str(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::ColorQuantize(_) => "color_quantize",
            Self::ColorRamp(_) => "color_ramp",
            Self::Crt(_) => "crt",
            Self::Downscale(_) => "downscale",
            Self::DirtyBloom(_) => "dirty_bloom",
            Self::FilmNoise(_) => "film_noise",
            Self::LensDroplets(_) => "lens_droplets",
            Self::RainGlass(_) => "rain_glass",
            Self::ShutterBlur(_) => "shutter_blur",
            Self::WetReflections(_) => "wet_reflections",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Downscale2dDocument {
    pub id: String,
    #[serde(default = "default_downscale_factor")]
    pub factor: f32,
    #[serde(default = "default_one")]
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShutterBlur2dDocument {
    pub id: String,
    #[serde(default = "default_shutter_blur_fps")]
    pub fps: f32,
    #[serde(default = "default_shutter_blur_shutter_angle")]
    pub shutter_angle: f32,
    #[serde(default = "default_shutter_blur_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub history_mix: f32,
    #[serde(default)]
    pub history_mix_2: f32,
    #[serde(default = "default_shutter_blur_edge_rejection")]
    pub edge_rejection: f32,
    #[serde(default = "default_shutter_blur_luma_threshold")]
    pub luma_threshold: f32,
    #[serde(default)]
    pub frame_hold: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorQuantize2dDocument {
    pub id: String,
    #[serde(default = "default_color_quantize_palette_size")]
    pub palette_size: u32,
    #[serde(default = "default_color_quantize_dither_strength")]
    pub dither_strength: f32,
    #[serde(default = "default_color_quantize_dither_scale")]
    pub dither_scale: f32,
    #[serde(default)]
    pub layered_dither: f32,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default = "default_color_quantize_luma_preserve")]
    pub luma_preserve: f32,
    #[serde(default)]
    pub highlight_bias: f32,
    #[serde(default)]
    pub shadow_bias: f32,
    #[serde(default = "default_one")]
    pub contrast: f32,
    #[serde(default = "default_one")]
    pub saturation: f32,
    #[serde(default = "default_color_quantize_gamma")]
    pub gamma: f32,
    #[serde(default = "default_color_quantize_seed")]
    pub seed: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorRamp2dDocument {
    pub id: String,
    #[serde(default = "default_color_ramp_palette_size")]
    pub palette_size: u32,
    #[serde(default = "default_color_ramp_dither_strength")]
    pub dither_strength: f32,
    #[serde(default = "default_color_quantize_dither_scale")]
    pub dither_scale: f32,
    #[serde(default = "default_color_ramp_layered_dither")]
    pub layered_dither: f32,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default = "default_color_ramp_luma_preserve")]
    pub luma_preserve: f32,
    #[serde(default = "default_color_ramp_highlight_bias")]
    pub highlight_bias: f32,
    #[serde(default = "default_color_ramp_shadow_bias")]
    pub shadow_bias: f32,
    #[serde(default = "default_color_ramp_contrast")]
    pub contrast: f32,
    #[serde(default = "default_color_ramp_saturation")]
    pub saturation: f32,
    #[serde(default = "default_color_ramp_gamma")]
    pub gamma: f32,
    #[serde(default = "default_color_quantize_seed")]
    pub seed: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirtyBloom2dDocument {
    pub id: String,
    #[serde(default = "default_dirty_bloom_threshold")]
    pub threshold: f32,
    #[serde(default = "default_dirty_bloom_strength")]
    pub strength: f32,
    #[serde(default = "default_dirty_bloom_small_radius_px")]
    pub small_radius_px: f32,
    #[serde(default = "default_dirty_bloom_medium_radius_px")]
    pub medium_radius_px: f32,
    #[serde(default = "default_dirty_bloom_large_radius_px")]
    pub large_radius_px: f32,
    #[serde(default = "default_dirty_bloom_dirty_noise")]
    pub dirty_noise: f32,
    #[serde(default = "default_dirty_bloom_halation_strength")]
    pub halation_strength: f32,
    #[serde(default = "default_dirty_bloom_reflection_smear_x_px")]
    pub reflection_smear_x_px: f32,
    #[serde(default = "default_dirty_bloom_reflection_smear_y_px")]
    pub reflection_smear_y_px: f32,
    #[serde(default = "default_dirty_bloom_seed")]
    pub seed: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crt2dDocument {
    pub id: String,
    #[serde(default = "default_crt_scanline_opacity")]
    pub scanline_opacity: f32,
    #[serde(default = "default_crt_scanline_frequency_px")]
    pub scanline_frequency_px: f32,
    #[serde(default = "default_crt_rgb_split_px")]
    pub rgb_split_px: f32,
    #[serde(default = "default_crt_curvature")]
    pub curvature: f32,
    #[serde(default = "default_crt_vignette")]
    pub vignette: f32,
    #[serde(default = "default_crt_phosphor_mask")]
    pub phosphor_mask: f32,
    #[serde(default = "default_crt_brightness_compensation")]
    pub brightness_compensation: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmNoise2dDocument {
    pub id: String,
    #[serde(default = "default_film_noise_iso")]
    pub iso: f32,
    #[serde(default = "default_film_noise_grain_size")]
    pub grain_size: f32,
    #[serde(default = "default_film_noise_chroma_noise")]
    pub chroma_noise: f32,
    #[serde(default = "default_film_noise_color_shift")]
    pub color_shift: f32,
    #[serde(default = "default_film_noise_contrast")]
    pub contrast: f32,
    #[serde(default = "default_film_noise_saturation")]
    pub saturation: f32,
    #[serde(default = "default_film_noise_flicker")]
    pub flicker: f32,
    #[serde(default = "default_film_noise_vignette")]
    pub vignette: f32,
    #[serde(default = "default_film_noise_toe")]
    pub toe: f32,
    #[serde(default = "default_film_noise_shoulder")]
    pub shoulder: f32,
    #[serde(default = "default_film_noise_black_lift")]
    pub black_lift: f32,
    #[serde(default = "default_film_noise_print_fade")]
    pub print_fade: f32,
    #[serde(default)]
    pub dust: f32,
    #[serde(default)]
    pub scratches: f32,
    #[serde(default)]
    pub push_pull: f32,
    #[serde(default)]
    pub gate_weave: f32,
    #[serde(default)]
    pub scan_softness: f32,
    #[serde(default = "default_film_noise_opacity")]
    pub opacity: f32,
    #[serde(default = "default_film_noise_seed")]
    pub seed: u32,
}

fn default_film_noise_toe() -> f32 {
    0.45
}

fn default_film_noise_shoulder() -> f32 {
    0.65
}

fn default_film_noise_black_lift() -> f32 {
    0.02
}

fn default_film_noise_print_fade() -> f32 {
    0.08
}
