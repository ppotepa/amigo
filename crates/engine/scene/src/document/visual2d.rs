use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}

fn default_white_color() -> String {
    "#ffffff".to_owned()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneVisual2dDocument {
    #[serde(default)]
    pub render_layers: Vec<RenderLayer2dDocument>,
    #[serde(default)]
    pub light_groups: Vec<LightGroup2dDocument>,
    #[serde(default)]
    pub light_routes: Vec<LightRoute2dDocument>,
    #[serde(default)]
    pub post_fx: Vec<PostFx2dDocument>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PostFx2dDocument {
    ColorQuantize(ColorQuantize2dDocument),
    Crt(Crt2dDocument),
    Downscale(Downscale2dDocument),
    DirtyBloom(DirtyBloom2dDocument),
    FilmNoise(FilmNoise2dDocument),
    LensDroplets(LensDroplets2dDocument),
    RainGlass(RainGlass2dDocument),
    ShutterBlur(ShutterBlur2dDocument),
    WetReflections(WetReflections2dDocument),
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
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default = "default_color_quantize_luma_preserve")]
    pub luma_preserve: f32,
    #[serde(default)]
    pub highlight_bias: f32,
    #[serde(default = "default_color_quantize_gamma")]
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
    #[serde(default = "default_film_noise_opacity")]
    pub opacity: f32,
    #[serde(default = "default_film_noise_seed")]
    pub seed: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensDroplets2dDocument {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub certification: LensDropletsCertificationDocument,
    #[serde(default)]
    pub affects: LensDropletsAffectsDocument,
    #[serde(default)]
    pub surface: LensDropletsSurfaceDocument,
    #[serde(default)]
    pub droplets: LensDropletsSpawnDocument,
    #[serde(default)]
    pub streaks: LensDropletsStreaksDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlass2dDocument {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rain_glass_spawn_rate")]
    pub spawn_rate: f32,
    #[serde(default = "default_rain_glass_spawn_limit")]
    pub spawn_limit: u32,
    #[serde(default = "default_rain_glass_spawn_size")]
    pub spawn_size: [f32; 2],
    #[serde(default = "default_rain_glass_refract_base")]
    pub refract_base: f32,
    #[serde(default = "default_rain_glass_refract_scale")]
    pub refract_scale: f32,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default = "default_rain_glass_light_bump")]
    pub light_bump: f32,
    #[serde(default = "default_rain_glass_seed")]
    pub seed: u32,
    #[serde(default)]
    pub simulation: RainGlassSimulationDocument,
    #[serde(default)]
    pub trails: RainGlassTrailsDocument,
    #[serde(default)]
    pub micro_droplets: RainGlassMicroDropletsDocument,
    #[serde(default)]
    pub mist: RainGlassMistDocument,
    #[serde(default)]
    pub render: RainGlassRenderDocument,
    #[serde(default)]
    pub lighting: RainGlassLightingDocument,
    #[serde(default)]
    pub debug: RainGlassDebugDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassSimulationDocument {
    #[serde(default = "default_rain_glass_gravity_px_per_sec2")]
    pub gravity_px_per_sec2: f32,
    #[serde(default = "default_rain_glass_slip_rate")]
    pub slip_rate: f32,
    #[serde(default = "default_rain_glass_motion_interval")]
    pub motion_interval: [f32; 2],
    #[serde(default = "default_rain_glass_x_shifting")]
    pub x_shifting: [f32; 2],
    #[serde(default = "default_rain_glass_collider_scale")]
    pub collider_scale: f32,
    #[serde(default = "default_rain_glass_initial_spread")]
    pub initial_spread: f32,
    #[serde(default = "default_rain_glass_shrink_rate")]
    pub shrink_rate: f32,
    #[serde(default = "default_rain_glass_velocity_spread")]
    pub velocity_spread: f32,
    #[serde(default = "default_rain_glass_evaporate")]
    pub evaporate: f32,
}

impl Default for RainGlassSimulationDocument {
    fn default() -> Self {
        Self {
            gravity_px_per_sec2: default_rain_glass_gravity_px_per_sec2(),
            slip_rate: default_rain_glass_slip_rate(),
            motion_interval: default_rain_glass_motion_interval(),
            x_shifting: default_rain_glass_x_shifting(),
            collider_scale: default_rain_glass_collider_scale(),
            initial_spread: default_rain_glass_initial_spread(),
            shrink_rate: default_rain_glass_shrink_rate(),
            velocity_spread: default_rain_glass_velocity_spread(),
            evaporate: default_rain_glass_evaporate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassTrailsDocument {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rain_glass_trail_density")]
    pub density: f32,
    #[serde(default = "default_rain_glass_trail_size")]
    pub size: [f32; 2],
    #[serde(default = "default_rain_glass_trail_distance_px")]
    pub distance_px: [f32; 2],
    #[serde(default = "default_rain_glass_trail_spread")]
    pub spread: f32,
    #[serde(default = "default_rain_glass_trail_shrink_rate")]
    pub shrink_rate: f32,
    #[serde(default = "default_rain_glass_trail_evaporate")]
    pub evaporate: f32,
    #[serde(default = "default_rain_glass_trail_taper")]
    pub taper: f32,
    #[serde(default = "default_rain_glass_streak_boost")]
    pub streak_boost: f32,
    #[serde(default = "default_rain_glass_streak_length")]
    pub streak_length: f32,
}

impl Default for RainGlassTrailsDocument {
    fn default() -> Self {
        Self {
            enabled: true,
            density: default_rain_glass_trail_density(),
            size: default_rain_glass_trail_size(),
            distance_px: default_rain_glass_trail_distance_px(),
            spread: default_rain_glass_trail_spread(),
            shrink_rate: default_rain_glass_trail_shrink_rate(),
            evaporate: default_rain_glass_trail_evaporate(),
            taper: default_rain_glass_trail_taper(),
            streak_boost: default_rain_glass_streak_boost(),
            streak_length: default_rain_glass_streak_length(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassMicroDropletsDocument {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rain_glass_micro_droplets_per_second")]
    pub per_second: f32,
    #[serde(default = "default_rain_glass_micro_droplet_size")]
    pub size: [f32; 2],
}

impl Default for RainGlassMicroDropletsDocument {
    fn default() -> Self {
        Self {
            enabled: true,
            per_second: default_rain_glass_micro_droplets_per_second(),
            size: default_rain_glass_micro_droplet_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassMistDocument {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rain_glass_mist_opacity")]
    pub opacity: f32,
    #[serde(default = "default_rain_glass_mist_blur_px")]
    pub blur_px: f32,
    #[serde(default = "default_rain_glass_mist_accumulation")]
    pub accumulation: f32,
    #[serde(default = "default_rain_glass_mist_time")]
    pub time: f32,
    #[serde(default = "default_rain_glass_mist_color_strength")]
    pub color_strength: f32,
    #[serde(default = "default_rain_glass_mist_blur_step")]
    pub blur_step: u32,
}

impl Default for RainGlassMistDocument {
    fn default() -> Self {
        Self {
            enabled: true,
            opacity: default_rain_glass_mist_opacity(),
            blur_px: default_rain_glass_mist_blur_px(),
            accumulation: default_rain_glass_mist_accumulation(),
            time: default_rain_glass_mist_time(),
            color_strength: default_rain_glass_mist_color_strength(),
            blur_step: default_rain_glass_mist_blur_step(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassRenderDocument {
    #[serde(default = "default_rain_glass_background_blur_px")]
    pub background_blur_px: f32,
    #[serde(default = "default_rain_glass_background_blur_steps")]
    pub background_blur_steps: u32,
    #[serde(default = "default_rain_glass_smooth_edge")]
    pub smooth_edge: [f32; 2],
    #[serde(default)]
    pub chromatic_aberration: f32,
    #[serde(default = "default_rain_glass_distortion_px")]
    pub distortion_px: f32,
    #[serde(default = "default_rain_glass_normal_strength")]
    pub normal_strength: f32,
    #[serde(default = "default_rain_glass_focus_blur_strength")]
    pub focus_blur_strength: f32,
    #[serde(default = "default_rain_glass_body_opacity")]
    pub body_opacity: f32,
    #[serde(default = "default_rain_glass_trail_refract_scale")]
    pub trail_refract_scale: f32,
    #[serde(default = "default_rain_glass_trail_opacity")]
    pub trail_opacity: f32,
    #[serde(default = "default_rain_glass_reference_mode")]
    pub reference_mode: bool,
    #[serde(default = "default_rain_glass_raindrop_compose")]
    pub raindrop_compose: String,
    #[serde(default = "default_rain_glass_raindrop_eraser_size")]
    pub raindrop_eraser_size: [f32; 2],
}

impl Default for RainGlassRenderDocument {
    fn default() -> Self {
        Self {
            background_blur_px: default_rain_glass_background_blur_px(),
            background_blur_steps: default_rain_glass_background_blur_steps(),
            smooth_edge: default_rain_glass_smooth_edge(),
            chromatic_aberration: 0.0,
            distortion_px: default_rain_glass_distortion_px(),
            normal_strength: default_rain_glass_normal_strength(),
            focus_blur_strength: default_rain_glass_focus_blur_strength(),
            body_opacity: default_rain_glass_body_opacity(),
            trail_refract_scale: default_rain_glass_trail_refract_scale(),
            trail_opacity: default_rain_glass_trail_opacity(),
            reference_mode: default_rain_glass_reference_mode(),
            raindrop_compose: default_rain_glass_raindrop_compose(),
            raindrop_eraser_size: default_rain_glass_raindrop_eraser_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassLightingDocument {
    #[serde(default = "default_rain_glass_light_pos")]
    pub light_pos: [f32; 4],
    #[serde(default = "default_rain_glass_diffuse")]
    pub diffuse: [f32; 3],
    #[serde(default = "default_rain_glass_shadow_offset")]
    pub shadow_offset: f32,
    #[serde(default = "default_rain_glass_specular")]
    pub specular: [f32; 3],
    #[serde(default = "default_rain_glass_specular_shininess")]
    pub specular_shininess: f32,
    #[serde(default = "default_rain_glass_scene_light_response")]
    pub scene_light_response: f32,
    #[serde(default = "default_rain_glass_rim_strength")]
    pub rim_strength: f32,
}

impl Default for RainGlassLightingDocument {
    fn default() -> Self {
        Self {
            light_pos: default_rain_glass_light_pos(),
            diffuse: default_rain_glass_diffuse(),
            shadow_offset: default_rain_glass_shadow_offset(),
            specular: default_rain_glass_specular(),
            specular_shininess: default_rain_glass_specular_shininess(),
            scene_light_response: default_rain_glass_scene_light_response(),
            rim_strength: default_rain_glass_rim_strength(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RainGlassDebugDocument {
    #[serde(default)]
    pub view: Option<String>,
}

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LensDropletsCertificationDocument {
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub budget: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensDropletsAffectsDocument {
    #[serde(default = "default_true")]
    pub world: bool,
    #[serde(default)]
    pub game_ui: bool,
    #[serde(default)]
    pub debug_ui: bool,
}

impl Default for LensDropletsAffectsDocument {
    fn default() -> Self {
        Self {
            world: true,
            game_ui: false,
            debug_ui: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensDropletsSurfaceDocument {
    #[serde(default)]
    pub dirt_opacity: f32,
    #[serde(default)]
    pub darken: f32,
    #[serde(default)]
    pub blur_px: f32,
    #[serde(default)]
    pub blur_samples: u32,
    #[serde(default)]
    pub distortion: f32,
    #[serde(default = "default_one")]
    pub downsample: f32,
}

impl Default for LensDropletsSurfaceDocument {
    fn default() -> Self {
        Self {
            dirt_opacity: 0.16,
            darken: 0.08,
            blur_px: 3.0,
            blur_samples: 4,
            distortion: 0.015,
            downsample: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensDropletsSpawnDocument {
    #[serde(default = "default_lens_droplets_max")]
    pub max: u32,
    #[serde(default = "default_lens_droplets_spawn_rate")]
    pub spawn_rate: f32,
    #[serde(default = "default_lens_droplets_radius_range")]
    pub radius_range: [f32; 2],
    #[serde(default = "default_lens_droplets_opacity_range")]
    pub opacity_range: [f32; 2],
    #[serde(default = "default_lens_droplets_lifetime_range")]
    pub lifetime_range: [f32; 2],
}

impl Default for LensDropletsSpawnDocument {
    fn default() -> Self {
        Self {
            max: 48,
            spawn_rate: 0.25,
            radius_range: [10.0, 42.0],
            opacity_range: [0.18, 0.52],
            lifetime_range: [4.0, 12.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensDropletsStreaksDocument {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_lens_droplets_streak_chance")]
    pub chance: f32,
    #[serde(default = "default_lens_droplets_gravity")]
    pub gravity_px_per_sec: f32,
    #[serde(default = "default_lens_droplets_max_streak_length")]
    pub max_length: f32,
    #[serde(default = "default_lens_droplets_wobble")]
    pub wobble: f32,
}

impl Default for LensDropletsStreaksDocument {
    fn default() -> Self {
        Self {
            enabled: true,
            chance: 0.16,
            gravity_px_per_sec: 24.0,
            max_length: 160.0,
            wobble: 0.35,
        }
    }
}

fn default_lens_droplets_max() -> u32 {
    48
}

fn default_lens_droplets_spawn_rate() -> f32 {
    0.25
}

fn default_lens_droplets_radius_range() -> [f32; 2] {
    [10.0, 42.0]
}

fn default_lens_droplets_opacity_range() -> [f32; 2] {
    [0.18, 0.52]
}

fn default_lens_droplets_lifetime_range() -> [f32; 2] {
    [4.0, 12.0]
}

fn default_lens_droplets_streak_chance() -> f32 {
    0.16
}

fn default_lens_droplets_gravity() -> f32 {
    24.0
}

fn default_lens_droplets_max_streak_length() -> f32 {
    160.0
}

fn default_lens_droplets_wobble() -> f32 {
    0.35
}

fn default_wet_blur_px() -> f32 {
    1.5
}

fn default_wet_distortion_px() -> f32 {
    0.8
}

fn default_wet_shimmer_strength() -> f32 {
    0.04
}

fn default_wet_ripple_strength() -> f32 {
    0.02
}

fn default_wet_darken() -> f32 {
    0.06
}

fn default_wet_specular_boost() -> f32 {
    0.25
}

fn default_wet_edge_power() -> f32 {
    1.35
}

fn default_wet_light_reflection_strength() -> f32 {
    0.65
}

fn default_wet_foreground_strength() -> f32 {
    1.0
}

fn default_wet_background_strength() -> f32 {
    0.12
}

fn default_wet_horizon_y() -> f32 {
    0.42
}

fn default_wet_noise_scale() -> f32 {
    2.5
}

fn default_wet_noise_speed() -> f32 {
    0.035
}

fn default_wet_ripple_speed() -> f32 {
    0.08
}

fn default_film_noise_iso() -> f32 {
    800.0
}

fn default_film_noise_grain_size() -> f32 {
    1.0
}

fn default_film_noise_chroma_noise() -> f32 {
    0.04
}

fn default_film_noise_color_shift() -> f32 {
    0.03
}

fn default_film_noise_contrast() -> f32 {
    1.0
}

fn default_film_noise_saturation() -> f32 {
    1.0
}

fn default_film_noise_flicker() -> f32 {
    0.12
}

fn default_film_noise_vignette() -> f32 {
    0.08
}

fn default_film_noise_opacity() -> f32 {
    0.35
}

fn default_film_noise_seed() -> u32 {
    1337
}

fn default_color_quantize_palette_size() -> u32 {
    64
}

fn default_color_quantize_dither_strength() -> f32 {
    0.35
}

fn default_color_quantize_luma_preserve() -> f32 {
    0.2
}

fn default_color_quantize_gamma() -> f32 {
    2.2
}

fn default_color_quantize_seed() -> u32 {
    911
}

fn default_downscale_factor() -> f32 {
    2.0
}

fn default_shutter_blur_fps() -> f32 {
    24.0
}

fn default_shutter_blur_shutter_angle() -> f32 {
    180.0
}

fn default_shutter_blur_opacity() -> f32 {
    0.72
}

fn default_shutter_blur_edge_rejection() -> f32 {
    0.35
}

fn default_shutter_blur_luma_threshold() -> f32 {
    0.04
}

fn default_dirty_bloom_threshold() -> f32 {
    0.62
}

fn default_dirty_bloom_strength() -> f32 {
    0.75
}

fn default_dirty_bloom_small_radius_px() -> f32 {
    3.0
}

fn default_dirty_bloom_medium_radius_px() -> f32 {
    12.0
}

fn default_dirty_bloom_large_radius_px() -> f32 {
    32.0
}

fn default_dirty_bloom_dirty_noise() -> f32 {
    0.18
}

fn default_dirty_bloom_halation_strength() -> f32 {
    0.22
}

fn default_dirty_bloom_reflection_smear_x_px() -> f32 {
    6.0
}

fn default_dirty_bloom_reflection_smear_y_px() -> f32 {
    28.0
}

fn default_dirty_bloom_seed() -> u32 {
    4242
}

fn default_crt_scanline_opacity() -> f32 {
    0.12
}

fn default_crt_scanline_frequency_px() -> f32 {
    1.5
}

fn default_crt_rgb_split_px() -> f32 {
    1.0
}

fn default_crt_curvature() -> f32 {
    0.03
}

fn default_crt_vignette() -> f32 {
    0.22
}

fn default_crt_phosphor_mask() -> f32 {
    0.04
}

fn default_crt_brightness_compensation() -> f32 {
    1.05
}

fn default_rain_glass_spawn_rate() -> f32 {
    10.0
}

fn default_rain_glass_spawn_limit() -> u32 {
    850
}

fn default_rain_glass_spawn_size() -> [f32; 2] {
    [12.0, 72.0]
}

fn default_rain_glass_refract_base() -> f32 {
    0.34
}

fn default_rain_glass_refract_scale() -> f32 {
    0.76
}

fn default_rain_glass_light_bump() -> f32 {
    0.78
}

fn default_rain_glass_seed() -> u32 {
    121713
}

fn default_rain_glass_gravity_px_per_sec2() -> f32 {
    2400.0
}

fn default_rain_glass_slip_rate() -> f32 {
    0.34
}

fn default_rain_glass_motion_interval() -> [f32; 2] {
    [0.10, 0.40]
}

fn default_rain_glass_x_shifting() -> [f32; 2] {
    [0.0, 0.08]
}

fn default_rain_glass_collider_scale() -> f32 {
    1.0
}

fn default_rain_glass_initial_spread() -> f32 {
    0.52
}

fn default_rain_glass_shrink_rate() -> f32 {
    0.014
}

fn default_rain_glass_velocity_spread() -> f32 {
    0.34
}

fn default_rain_glass_evaporate() -> f32 {
    11.0
}

fn default_rain_glass_trail_density() -> f32 {
    0.20
}

fn default_rain_glass_trail_size() -> [f32; 2] {
    [0.28, 0.48]
}

fn default_rain_glass_trail_distance_px() -> [f32; 2] {
    [18.0, 36.0]
}

fn default_rain_glass_trail_spread() -> f32 {
    0.58
}

fn default_rain_glass_trail_shrink_rate() -> f32 {
    0.975
}

fn default_rain_glass_trail_evaporate() -> f32 {
    18.0
}

fn default_rain_glass_trail_taper() -> f32 {
    0.68
}

fn default_rain_glass_streak_boost() -> f32 {
    0.72
}

fn default_rain_glass_streak_length() -> f32 {
    1.15
}

fn default_rain_glass_micro_droplets_per_second() -> f32 {
    620.0
}

fn default_rain_glass_micro_droplet_size() -> [f32; 2] {
    [8.0, 27.0]
}

fn default_rain_glass_mist_opacity() -> f32 {
    1.0
}

fn default_rain_glass_mist_blur_px() -> f32 {
    4.0
}

fn default_rain_glass_mist_accumulation() -> f32 {
    0.012
}

fn default_rain_glass_mist_time() -> f32 {
    16.0
}

fn default_rain_glass_mist_color_strength() -> f32 {
    0.012
}

fn default_rain_glass_mist_blur_step() -> u32 {
    4
}

fn default_rain_glass_background_blur_px() -> f32 {
    2.0
}

fn default_rain_glass_background_blur_steps() -> u32 {
    2
}

fn default_rain_glass_smooth_edge() -> [f32; 2] {
    [0.945, 0.992]
}

fn default_rain_glass_distortion_px() -> f32 {
    28.0
}

fn default_rain_glass_normal_strength() -> f32 {
    6.0
}

fn default_rain_glass_focus_blur_strength() -> f32 {
    0.85
}

fn default_rain_glass_body_opacity() -> f32 {
    0.92
}

fn default_rain_glass_trail_refract_scale() -> f32 {
    0.48
}

fn default_rain_glass_trail_opacity() -> f32 {
    0.72
}

fn default_rain_glass_reference_mode() -> bool {
    true
}

fn default_rain_glass_raindrop_compose() -> String {
    "smoother".to_owned()
}

fn default_rain_glass_raindrop_eraser_size() -> [f32; 2] {
    [0.93, 1.0]
}

fn default_rain_glass_light_pos() -> [f32; 4] {
    [-1.0, 1.0, 2.0, 0.0]
}

fn default_rain_glass_diffuse() -> [f32; 3] {
    [0.035, 0.045, 0.055]
}

fn default_rain_glass_shadow_offset() -> f32 {
    0.76
}

fn default_rain_glass_specular() -> [f32; 3] {
    [0.018, 0.022, 0.028]
}

fn default_rain_glass_specular_shininess() -> f32 {
    300.0
}

fn default_rain_glass_scene_light_response() -> f32 {
    1.45
}

fn default_rain_glass_rim_strength() -> f32 {
    1.15
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLayer2dDocument {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub order: f32,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_one")]
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightRoute2dDocument {
    pub receiver_layer: String,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightGroup2dDocument {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_white_color")]
    pub color: String,
    #[serde(default = "default_one")]
    pub intensity: f32,
    #[serde(default)]
    pub sources: Vec<LightGroup2dSourceDocument>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LightGroup2dSourceDocument {
    LightmapChannel {
        source: String,
        channel: String,
        #[serde(default = "default_one")]
        response: f32,
    },
    GlobalLight {
        id: String,
        #[serde(default = "default_one")]
        response: f32,
    },
}
