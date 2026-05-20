use super::*;

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
    pub depth: RainGlassDepthDocument,
    #[serde(default)]
    pub debug: RainGlassDebugDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RainGlassDepthDocument {
    #[serde(default)]
    pub z_depth: Option<f32>,
    #[serde(default = "default_rain_glass_z_depth_blur_scale")]
    pub blur_scale: f32,
    #[serde(default)]
    pub focus_response: f32,
}

impl Default for RainGlassDepthDocument {
    fn default() -> Self {
        Self {
            z_depth: None,
            blur_scale: default_rain_glass_z_depth_blur_scale(),
            focus_response: 0.0,
        }
    }
}

fn default_rain_glass_z_depth_blur_scale() -> f32 {
    1.0
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
    #[serde(default = "default_rain_glass_scene_blend")]
    pub scene_blend: f32,
    #[serde(default)]
    pub drop_plane_blur_px: f32,
    #[serde(default = "default_rain_glass_receives_scene_light")]
    pub receives_scene_light: bool,
    #[serde(default = "default_rain_glass_scene_light_tint_strength")]
    pub scene_light_tint_strength: f32,
    #[serde(default = "default_rain_glass_scene_shadow_floor")]
    pub scene_shadow_floor: f32,
    #[serde(default = "default_rain_glass_blood_tint")]
    pub blood_tint: [f32; 3],
    #[serde(default = "default_rain_glass_blood_amount")]
    pub blood_amount: f32,
    #[serde(default = "default_rain_glass_scene_darken")]
    pub scene_darken: f32,
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
            scene_blend: default_rain_glass_scene_blend(),
            drop_plane_blur_px: 0.0,
            receives_scene_light: default_rain_glass_receives_scene_light(),
            scene_light_tint_strength: default_rain_glass_scene_light_tint_strength(),
            scene_shadow_floor: default_rain_glass_scene_shadow_floor(),
            blood_tint: default_rain_glass_blood_tint(),
            blood_amount: default_rain_glass_blood_amount(),
            scene_darken: default_rain_glass_scene_darken(),
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
