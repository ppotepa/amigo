use std::collections::BTreeMap;

use amigo_camera_optics_plugin::scene::CameraOpticalResponse2dSceneCommand;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthSpace2dSceneCommand {
    pub near_m: f32,
    pub far_m: f32,
    pub curve: DepthCurve2dSceneCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthCurve2dSceneCommand {
    Linear,
    Logarithmic,
}

impl DepthSpace2dSceneCommand {
    pub fn to_runtime(self) -> amigo_2d_spatial::DepthSpace2d {
        amigo_2d_spatial::DepthSpace2d {
            near_m: self.near_m,
            far_m: self.far_m,
            curve: match self.curve {
                DepthCurve2dSceneCommand::Linear => amigo_2d_spatial::DepthCurve2d::Linear,
                DepthCurve2dSceneCommand::Logarithmic => {
                    amigo_2d_spatial::DepthCurve2d::Logarithmic
                }
            },
        }
        .normalized()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteSheet2dSceneCommand {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpriteAnimation2dSceneOverride {
    pub fps: Option<f32>,
    pub looping: Option<bool>,
    pub start_frame: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageBlendMode2dSceneCommand {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageViewportFit2dSceneCommand {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VisualMaps2dSceneCommand {
    pub normal: Option<AssetKey>,
    pub wetness: Option<AssetKey>,
    pub emissive: Option<AssetKey>,
    pub highlight: Option<AssetKey>,
    pub roughness: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayerOverrideSceneCommand {
    pub id: String,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<LayeredImageBlendMode2dSceneCommand>,
    pub visual_maps: Option<VisualMaps2dSceneCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImage2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub render_layer: String,
    pub asset: AssetKey,
    pub size: Vec2,
    pub base_opacity: f32,
    pub viewport_fit: LayeredImageViewportFit2dSceneCommand,
    pub visual_maps: Option<VisualMaps2dSceneCommand>,
    pub z_index: f32,
    pub transform: Transform2,
    pub layer_overrides: Vec<LayeredImageLayerOverrideSceneCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthMapViewportFit2dSceneCommand {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthMap2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub asset: AssetKey,
    pub size: Vec2,
    pub viewport_fit: DepthMapViewportFit2dSceneCommand,
    pub white_is_near: bool,
    pub z_index: f32,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthAuxMap2dChannelsSceneCommand {
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

impl Default for DepthAuxMap2dChannelsSceneCommand {
    fn default() -> Self {
        Self {
            r: "auxiliary_depth".to_owned(),
            g: "local_height".to_owned(),
            b: "occluder_strength".to_owned(),
            a: "valid_mask".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthAuxMap2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub asset: AssetKey,
    pub surface_asset: Option<AssetKey>,
    pub size: Vec2,
    pub viewport_fit: DepthMapViewportFit2dSceneCommand,
    pub channels: DepthAuxMap2dChannelsSceneCommand,
    pub z_index: f32,
    pub transform: Transform2,
}

impl LayeredImage2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        asset: AssetKey,
        size: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            render_layer: "default".to_owned(),
            asset,
            size,
            base_opacity: 1.0,
            viewport_fit: LayeredImageViewportFit2dSceneCommand::Fixed,
            visual_maps: None,
            z_index: 0.0,
            transform: Transform2::default(),
            layer_overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMap2dSourceKindSceneCommand {
    LayeredImage2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dSourceRefSceneCommand {
    pub kind: LightMap2dSourceKindSceneCommand,
    pub entity_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dChannelSceneCommand {
    pub id: String,
    pub layers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dSourceSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub source: LightMap2dSourceRefSceneCommand,
    pub channels: Vec<LightMap2dChannelSceneCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLight2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub color: ColorRgba,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub camera_id: String,
    pub mode: CameraExposureMode2dSceneCommand,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub exposure: CameraExposure2dSceneCommand,
    pub shutter: CameraShutter2dSceneCommand,
    pub lens: CameraLens2dSceneCommand,
    pub lens_surface: CameraLensSurface2dSceneCommand,
    pub film: CameraFilm2dSceneCommand,
    pub look: CameraLook2dSceneCommand,
    pub aperture: CameraAperture2dSceneCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderContributions2dSceneCommand {
    pub roles: BTreeMap<String, bool>,
}

impl RenderContributions2dSceneCommand {
    pub fn enabled_or(&self, role: &str, default: bool) -> bool {
        self.roles.get(role).copied().unwrap_or(default)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraExposureMode2dSceneCommand {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraExposure2dSceneCommand {
    pub iso: f32,
    pub compensation: f32,
    pub white_balance: f32,
    pub nd_stops: f32,
    pub auto: CameraAutoExposure2dSceneCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraAutoExposure2dSceneCommand {
    pub target_luma: f32,
    pub adaptation_speed: f32,
    pub min_iso: f32,
    pub max_iso: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraShutter2dSceneCommand {
    pub enabled: bool,
    pub speed_s: Option<f32>,
    pub fps: f32,
    pub angle: f32,
    pub opacity: f32,
    pub history_mix: f32,
    pub history_mix_2: f32,
    pub edge_rejection: f32,
    pub luma_threshold: f32,
    pub frame_hold: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLens2dSceneCommand {
    pub profile: String,
    pub intensity: f32,
    pub aberration_px: Option<f32>,
    pub distortion: Option<f32>,
    pub vignette: Option<f32>,
    pub edge_softness_px: Option<f32>,
    pub glare_strength: Option<f32>,
    pub dirt: Option<f32>,
    pub focal_length_mm: Option<f32>,
    pub lens_bloom: Option<f32>,
    pub flare_ghosts: Option<f32>,
    pub anamorphic_squeeze: Option<f32>,
    pub coma: Option<f32>,
    pub cat_eye_bokeh: Option<f32>,
    pub focus_breathing: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLensSurface2dSceneCommand {
    pub rain_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFilm2dSceneCommand {
    pub profile: String,
    pub intensity: f32,
    pub seed: u32,
    pub color_shift: Option<f32>,
    pub contrast: Option<f32>,
    pub saturation: Option<f32>,
    pub flicker: Option<f32>,
    pub vignette: Option<f32>,
    pub toe: Option<f32>,
    pub shoulder: Option<f32>,
    pub black_lift: Option<f32>,
    pub print_fade: Option<f32>,
    pub dust: Option<f32>,
    pub scratches: Option<f32>,
    pub push_pull: Option<f32>,
    pub gate_weave: Option<f32>,
    pub scan_softness: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLook2dSceneCommand {
    pub profile: String,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraAperture2dSceneCommand {
    pub enabled: bool,
    pub f_stop: f32,
    pub focus_distance_m: f32,
    pub focus: CameraFocus2dSceneCommand,
    pub depth_of_field: CameraDepthOfField2dSceneCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDepthOfField2dSceneCommand {
    pub depth_map: Option<String>,
    pub affected_layers: Vec<String>,
    pub max_blur_px: f32,
    pub depth_contrast: f32,
    pub focus_width: f32,
    pub foreground_blur_boost: f32,
    pub background_blur_boost: f32,
    pub edge_aware: bool,
    pub invert_depth: bool,
    pub debug_view: String,
    pub aperture_blades: u32,
    pub aperture_roundness: f32,
    pub aperture_rotation_degrees: f32,
    pub sample_count: u32,
    pub highlight_threshold: f32,
    pub highlight_knee: f32,
    pub highlight_gain: f32,
    pub highlight_saturation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraFocus2dSceneCommand {
    None,
    RenderLayer { layer: String },
    SceneObject { object: String },
    Distance { distance_m: f32 },
    Depth { value: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLayer2dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub order: f32,
    pub visible: bool,
    pub opacity: f32,
    pub depth: RenderDepth2dSceneCommand,
    pub optical_role: OpticalLayerRole2dSceneCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpticalLayerRole2dSceneCommand {
    WorldSurface,
    SceneMedium,
    ForegroundMedium,
    LensSurface,
    Overlay,
    Debug,
}

impl OpticalLayerRole2dSceneCommand {
    pub fn to_runtime(self) -> amigo_2d_spatial::OpticalLayerRole2d {
        match self {
            Self::WorldSurface => amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
            Self::SceneMedium => amigo_2d_spatial::OpticalLayerRole2d::SceneMedium,
            Self::ForegroundMedium => amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium,
            Self::LensSurface => amigo_2d_spatial::OpticalLayerRole2d::LensSurface,
            Self::Overlay => amigo_2d_spatial::OpticalLayerRole2d::Overlay,
            Self::Debug => amigo_2d_spatial::OpticalLayerRole2d::Debug,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDepthMode2dSceneCommand {
    DepthMap,
    Distance,
    ZDepth,
    Infinity,
    Overlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepth2dSceneCommand {
    pub mode: RenderDepthMode2dSceneCommand,
    pub distance_m: Option<f32>,
    pub z_depth: f32,
    pub blur_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRoute2dSceneCommand {
    pub source_mod: String,
    pub receiver_layer: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightGroup2dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub color: ColorRgba,
    pub intensity: f32,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub camera_response: CameraOpticalResponse2dSceneCommand,
    pub sources: Vec<LightGroup2dSourceSceneCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightGroup2dSourceSceneCommand {
    pub kind: LightGroup2dSourceKindSceneCommand,
    pub response: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightGroup2dSourceKindSceneCommand {
    LightMapChannel { source: String, channel: String },
    GlobalLight { id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSampleStrategy2dSceneCommand {
    Point,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightReceiverDarkPolicy2dSceneCommand {
    Transparent,
    BaseColor,
    ShadowTint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiverGlobalLight2dSceneCommand {
    pub id: String,
    pub response: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiver2dBindingSceneCommand {
    pub groups: Vec<String>,
    pub source: String,
    pub channel: String,
    pub sample_strategy: LightSampleStrategy2dSceneCommand,
    pub sample_points: u32,
    pub radius_px: f32,
    pub exposure: f32,
    pub dark_policy: LightReceiverDarkPolicy2dSceneCommand,
    pub global_lights: Vec<LightReceiverGlobalLight2dSceneCommand>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub render_layer: String,
    pub texture: AssetKey,
    pub size: Vec2,
    pub sheet: Option<SpriteSheet2dSceneCommand>,
    pub animation: Option<SpriteAnimation2dSceneOverride>,
    pub visual_maps: Option<VisualMaps2dSceneCommand>,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub material: Option<Material2dSceneCommand>,
    pub z_index: f32,
    pub transform: Transform2,
}

impl Sprite2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        texture: AssetKey,
        size: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            render_layer: "default".to_owned(),
            texture,
            size,
            sheet: None,
            animation: None,
            visual_maps: None,
            render_contributions: RenderContributions2dSceneCommand::default(),
            material: None,
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub render_layer: String,
    pub tileset: AssetKey,
    pub ruleset: Option<AssetKey>,
    pub tile_size: Vec2,
    pub grid: Vec<String>,
    pub depth_fill_rows: usize,
    pub z_index: f32,
}

impl TileMap2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        tileset: AssetKey,
        tile_size: Vec2,
        grid: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            render_layer: "default".to_owned(),
            tileset,
            ruleset: None,
            tile_size,
            grid,
            depth_fill_rows: 0,
            z_index: 0.0,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Text2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub render_layer: String,
    pub content: String,
    pub font: AssetKey,
    pub bounds: Vec2,
    pub style: Text2dStyleSceneCommand,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub post_fx_host_id: Option<amigo_composite_plugin::PostFxHost2dId>,
    pub z_index: f32,
    pub material: Option<Material2dSceneCommand>,
    pub transform: Transform2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text2dStyleSceneCommand {
    pub color: ColorRgba,
    pub opacity: f32,
    pub font_size: Option<f32>,
    pub align: Text2dAlignSceneCommand,
    pub blend: Text2dBlendModeSceneCommand,
    pub shadow: Option<Text2dShadowSceneCommand>,
    pub outline: Option<Text2dOutlineSceneCommand>,
    pub glow: Option<Text2dGlowSceneCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text2dAlignSceneCommand {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text2dBlendModeSceneCommand {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text2dShadowSceneCommand {
    pub color: ColorRgba,
    pub offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text2dOutlineSceneCommand {
    pub color: ColorRgba,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text2dGlowSceneCommand {
    pub color: ColorRgba,
    pub radius: f32,
    pub intensity: f32,
    pub passes: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material2dOpticalModeSceneCommand {
    Opaque,
    Transmissive,
    Refractive,
    Emissive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dOpticalSceneCommand {
    pub mode: Material2dOpticalModeSceneCommand,
    pub transmission: f32,
    pub refraction_px: f32,
    pub distortion: f32,
    pub dispersion: f32,
    pub roughness: f32,
    pub edge_boost: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dLightingSceneCommand {
    pub receives_light: bool,
    pub response: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dSceneCommand {
    pub optical: Material2dOpticalSceneCommand,
    pub lighting: Material2dLightingSceneCommand,
    pub camera_response: CameraOpticalResponse2dSceneCommand,
}

impl Default for Text2dStyleSceneCommand {
    fn default() -> Self {
        Self {
            color: ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            opacity: 1.0,
            font_size: None,
            align: Text2dAlignSceneCommand::Left,
            blend: Text2dBlendModeSceneCommand::Alpha,
            shadow: None,
            outline: None,
            glow: None,
        }
    }
}

impl Text2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        content: impl Into<String>,
        font: AssetKey,
        bounds: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            render_layer: "default".to_owned(),
            content: content.into(),
            font,
            bounds,
            style: Text2dStyleSceneCommand::default(),
            render_contributions: RenderContributions2dSceneCommand::default(),
            post_fx_host_id: None,
            z_index: 0.0,
            material: None,
            transform: Transform2::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum VectorShapeKind2dSceneCommand {
    Polyline { points: Vec<Vec2>, closed: bool },
    Polygon { points: Vec<Vec2> },
    Circle { radius: f32, segments: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorStyle2dSceneCommand {
    pub stroke_color: ColorRgba,
    pub stroke_width: f32,
    pub fill_color: Option<ColorRgba>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub render_layer: String,
    pub kind: VectorShapeKind2dSceneCommand,
    pub style: VectorStyle2dSceneCommand,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub material: Option<Material2dSceneCommand>,
    pub z_index: f32,
    pub transform: Transform2,
}

impl VectorShape2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        kind: VectorShapeKind2dSceneCommand,
        style: VectorStyle2dSceneCommand,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            render_layer: "default".to_owned(),
            kind,
            style,
            render_contributions: RenderContributions2dSceneCommand::default(),
            material: None,
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconLight2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub render_layer: String,
    pub color: ColorRgba,
    pub base_intensity: f32,
    pub frequency_hz: f32,
    pub duty_cycle: f32,
    pub rise_seconds: f32,
    pub fall_seconds: f32,
    pub phase_offset: f32,
    pub sync_group: Option<String>,
    pub jitter_amount: f32,
    pub jitter_hz: f32,
    pub core_radius_px: f32,
    pub halo_radius_px: f32,
    pub glow_strength: f32,
    pub beam_enabled: bool,
    pub beam_length_px: f32,
    pub beam_width_degrees: f32,
    pub beam_strength: f32,
    pub aberration_px: f32,
    pub bloom: f32,
    pub camera_response: CameraOpticalResponse2dSceneCommand,
    pub depth: Option<RenderDepth2dSceneCommand>,
    pub z_depth: Option<f32>,
    pub z_index: f32,
    pub render_contributions: RenderContributions2dSceneCommand,
    pub enabled: bool,
    pub transform: Transform2,
    pub viewport_fit: LayeredImageViewportFit2dSceneCommand,
    pub viewport_canvas_size: Option<Vec2>,
}
