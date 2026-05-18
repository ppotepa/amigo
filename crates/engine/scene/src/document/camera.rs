use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Camera2dModeDocument {
    Auto,
    Manual,
}

impl Default for Camera2dModeDocument {
    fn default() -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraExposure2dDocument {
    #[serde(default = "default_iso")]
    pub iso: f32,

    #[serde(default)]
    pub compensation: f32,

    #[serde(default = "default_white_balance")]
    pub white_balance: f32,

    #[serde(default)]
    pub nd_stops: f32,

    #[serde(default)]
    pub auto: CameraAutoExposure2dDocument,
}

impl Default for CameraExposure2dDocument {
    fn default() -> Self {
        Self {
            iso: default_iso(),
            compensation: 0.0,
            white_balance: default_white_balance(),
            nd_stops: 0.0,
            auto: CameraAutoExposure2dDocument::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraAutoExposure2dDocument {
    #[serde(default = "default_target_luma")]
    pub target_luma: f32,

    #[serde(default = "default_adaptation_speed")]
    pub adaptation_speed: f32,

    #[serde(default = "default_min_iso")]
    pub min_iso: f32,

    #[serde(default = "default_max_iso")]
    pub max_iso: f32,
}

impl Default for CameraAutoExposure2dDocument {
    fn default() -> Self {
        Self {
            target_luma: default_target_luma(),
            adaptation_speed: default_adaptation_speed(),
            min_iso: default_min_iso(),
            max_iso: default_max_iso(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraShutter2dDocument {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_shutter_fps")]
    pub fps: f32,

    #[serde(default = "default_shutter_angle")]
    pub angle: f32,

    #[serde(default = "default_shutter_opacity")]
    pub opacity: f32,

    #[serde(default = "default_history_mix")]
    pub history_mix: f32,

    #[serde(default = "default_history_mix_2")]
    pub history_mix_2: f32,

    #[serde(default = "default_edge_rejection")]
    pub edge_rejection: f32,

    #[serde(default = "default_luma_threshold")]
    pub luma_threshold: f32,

    #[serde(default)]
    pub frame_hold: bool,
}

impl Default for CameraShutter2dDocument {
    fn default() -> Self {
        Self {
            enabled: true,
            fps: default_shutter_fps(),
            angle: default_shutter_angle(),
            opacity: default_shutter_opacity(),
            history_mix: default_history_mix(),
            history_mix_2: default_history_mix_2(),
            edge_rejection: default_edge_rejection(),
            luma_threshold: default_luma_threshold(),
            frame_hold: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraLens2dDocument {
    #[serde(default = "default_lens_profile")]
    pub profile: String,

    #[serde(default = "default_profile_intensity")]
    pub intensity: f32,

    #[serde(default)]
    pub aberration_px: Option<f32>,

    #[serde(default)]
    pub distortion: Option<f32>,

    #[serde(default)]
    pub vignette: Option<f32>,

    #[serde(default)]
    pub edge_softness_px: Option<f32>,

    #[serde(default)]
    pub flare_strength: Option<f32>,

    #[serde(default)]
    pub dirt: Option<f32>,

    #[serde(default)]
    pub focal_length_mm: Option<f32>,

    #[serde(default)]
    pub lens_bloom: Option<f32>,

    #[serde(default)]
    pub flare_ghosts: Option<f32>,

    #[serde(default)]
    pub anamorphic_squeeze: Option<f32>,

    #[serde(default)]
    pub coma: Option<f32>,

    #[serde(default)]
    pub cat_eye_bokeh: Option<f32>,

    #[serde(default)]
    pub focus_breathing: Option<f32>,
}

impl Default for CameraLens2dDocument {
    fn default() -> Self {
        Self {
            profile: default_lens_profile(),
            intensity: default_profile_intensity(),
            aberration_px: None,
            distortion: None,
            vignette: None,
            edge_softness_px: None,
            flare_strength: None,
            dirt: None,
            focal_length_mm: None,
            lens_bloom: None,
            flare_ghosts: None,
            anamorphic_squeeze: None,
            coma: None,
            cat_eye_bokeh: None,
            focus_breathing: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CameraLensSurface2dDocument {
    #[serde(default)]
    pub rain_profile: Option<String>,
}

impl Default for CameraLensSurface2dDocument {
    fn default() -> Self {
        Self { rain_profile: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraFilm2dDocument {
    #[serde(default = "default_film_profile")]
    pub profile: String,

    #[serde(default = "default_profile_intensity")]
    pub intensity: f32,

    #[serde(default)]
    pub seed: u32,

    #[serde(default)]
    pub color_shift: Option<f32>,

    #[serde(default)]
    pub contrast: Option<f32>,

    #[serde(default)]
    pub saturation: Option<f32>,

    #[serde(default)]
    pub flicker: Option<f32>,

    #[serde(default)]
    pub vignette: Option<f32>,

    #[serde(default)]
    pub toe: Option<f32>,

    #[serde(default)]
    pub shoulder: Option<f32>,

    #[serde(default)]
    pub black_lift: Option<f32>,

    #[serde(default)]
    pub print_fade: Option<f32>,

    #[serde(default)]
    pub dust: Option<f32>,

    #[serde(default)]
    pub scratches: Option<f32>,

    #[serde(default)]
    pub push_pull: Option<f32>,

    #[serde(default)]
    pub gate_weave: Option<f32>,

    #[serde(default)]
    pub scan_softness: Option<f32>,
}

impl Default for CameraFilm2dDocument {
    fn default() -> Self {
        Self {
            profile: default_film_profile(),
            intensity: default_profile_intensity(),
            seed: 1337,
            color_shift: None,
            contrast: None,
            saturation: None,
            flicker: None,
            vignette: None,
            toe: None,
            shoulder: None,
            black_lift: None,
            print_fade: None,
            dust: None,
            scratches: None,
            push_pull: None,
            gate_weave: None,
            scan_softness: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraLook2dDocument {
    #[serde(default = "default_look_profile")]
    pub profile: String,

    #[serde(default = "default_profile_intensity")]
    pub intensity: f32,
}

impl Default for CameraLook2dDocument {
    fn default() -> Self {
        Self {
            profile: default_look_profile(),
            intensity: default_profile_intensity(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraAperture2dDocument {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_f_stop")]
    pub f_stop: f32,

    #[serde(default = "default_focus_distance_m")]
    pub focus_distance_m: f32,

    #[serde(default)]
    pub focus: CameraFocus2dDocument,

    #[serde(default)]
    pub depth_of_field: CameraDepthOfField2dDocument,
}

impl Default for CameraAperture2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            f_stop: default_f_stop(),
            focus_distance_m: default_focus_distance_m(),
            focus: CameraFocus2dDocument::default(),
            depth_of_field: CameraDepthOfField2dDocument::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraDepthOfField2dDocument {
    #[serde(default)]
    pub depth_map: Option<String>,

    #[serde(default)]
    pub affected_layers: Vec<String>,

    #[serde(default = "default_depth_dof_max_blur_px")]
    pub max_blur_px: f32,

    #[serde(default = "default_depth_dof_depth_contrast")]
    pub depth_contrast: f32,

    #[serde(default = "default_depth_dof_focus_width")]
    pub focus_width: f32,

    #[serde(default = "default_depth_dof_foreground_blur_boost")]
    pub foreground_blur_boost: f32,

    #[serde(default = "default_depth_dof_background_blur_boost")]
    pub background_blur_boost: f32,

    #[serde(default = "default_true")]
    pub edge_aware: bool,

    #[serde(default)]
    pub invert_depth: bool,

    #[serde(default = "default_focus_blur_debug_view")]
    pub debug_view: String,

    #[serde(default = "default_depth_dof_aperture_blades")]
    pub aperture_blades: u32,

    #[serde(default = "default_depth_dof_aperture_roundness")]
    pub aperture_roundness: f32,

    #[serde(default)]
    pub aperture_rotation_degrees: f32,

    #[serde(default = "default_depth_dof_sample_count")]
    pub sample_count: u32,

    #[serde(default = "default_depth_dof_highlight_threshold")]
    pub highlight_threshold: f32,

    #[serde(default = "default_depth_dof_highlight_knee")]
    pub highlight_knee: f32,

    #[serde(default = "default_depth_dof_highlight_gain")]
    pub highlight_gain: f32,

    #[serde(default = "default_depth_dof_highlight_saturation")]
    pub highlight_saturation: f32,
}

impl Default for CameraDepthOfField2dDocument {
    fn default() -> Self {
        Self {
            depth_map: None,
            affected_layers: Vec::new(),
            max_blur_px: default_depth_dof_max_blur_px(),
            depth_contrast: default_depth_dof_depth_contrast(),
            focus_width: default_depth_dof_focus_width(),
            foreground_blur_boost: default_depth_dof_foreground_blur_boost(),
            background_blur_boost: default_depth_dof_background_blur_boost(),
            edge_aware: true,
            invert_depth: false,
            debug_view: default_focus_blur_debug_view(),
            aperture_blades: default_depth_dof_aperture_blades(),
            aperture_roundness: default_depth_dof_aperture_roundness(),
            aperture_rotation_degrees: 0.0,
            sample_count: default_depth_dof_sample_count(),
            highlight_threshold: default_depth_dof_highlight_threshold(),
            highlight_knee: default_depth_dof_highlight_knee(),
            highlight_gain: default_depth_dof_highlight_gain(),
            highlight_saturation: default_depth_dof_highlight_saturation(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CameraFocus2dDocument {
    None,
    RenderLayer { layer: String },
    SceneObject { object: String },
    Distance { distance_m: f32 },
    Depth { value: f32 },
}

impl Default for CameraFocus2dDocument {
    fn default() -> Self {
        Self::None
    }
}

pub fn default_camera2d_id() -> String {
    "main".to_string()
}

fn default_true() -> bool {
    true
}

fn default_iso() -> f32 {
    800.0
}

fn default_white_balance() -> f32 {
    5600.0
}

fn default_target_luma() -> f32 {
    0.42
}

fn default_adaptation_speed() -> f32 {
    0.8
}

fn default_min_iso() -> f32 {
    100.0
}

fn default_max_iso() -> f32 {
    3200.0
}

fn default_shutter_fps() -> f32 {
    24.0
}

fn default_shutter_angle() -> f32 {
    180.0
}

fn default_shutter_opacity() -> f32 {
    0.72
}

fn default_history_mix() -> f32 {
    0.0
}

fn default_history_mix_2() -> f32 {
    0.0
}

fn default_edge_rejection() -> f32 {
    0.35
}

fn default_luma_threshold() -> f32 {
    0.04
}

fn default_lens_profile() -> String {
    "clean_modern_35mm".to_string()
}

fn default_film_profile() -> String {
    "neutral_digital_400".to_string()
}

fn default_look_profile() -> String {
    String::new()
}

fn default_profile_intensity() -> f32 {
    1.0
}

fn default_f_stop() -> f32 {
    8.0
}

fn default_focus_distance_m() -> f32 {
    5.0
}

fn default_depth_dof_max_blur_px() -> f32 {
    28.0
}

fn default_depth_dof_depth_contrast() -> f32 {
    1.0
}

fn default_depth_dof_focus_width() -> f32 {
    0.055
}

fn default_depth_dof_foreground_blur_boost() -> f32 {
    1.15
}

fn default_depth_dof_background_blur_boost() -> f32 {
    1.0
}

fn default_focus_blur_debug_view() -> String {
    "final".to_owned()
}

fn default_depth_dof_aperture_blades() -> u32 {
    7
}

fn default_depth_dof_aperture_roundness() -> f32 {
    0.72
}

fn default_depth_dof_sample_count() -> u32 {
    64
}

fn default_depth_dof_highlight_threshold() -> f32 {
    0.68
}

fn default_depth_dof_highlight_knee() -> f32 {
    0.18
}

fn default_depth_dof_highlight_gain() -> f32 {
    1.45
}

fn default_depth_dof_highlight_saturation() -> f32 {
    1.10
}
