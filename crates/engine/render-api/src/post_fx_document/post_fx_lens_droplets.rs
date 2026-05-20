use super::*;

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
