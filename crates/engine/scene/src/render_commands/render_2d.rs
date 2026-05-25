use std::collections::BTreeMap;
use std::sync::Arc;

use amigo_camera::CameraOpticalResponse2dSceneCommand;

use crate::{PluginSceneCommand, PluginSceneCommandPayload, SceneAssetDependency};

pub const VISUAL2D_SPATIAL_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.composition-2d.scene-command.Visual2dSpatial";
pub const RENDER_LAYER_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.composition-2d.scene-command.RenderLayer2d";
pub const LIGHT_ROUTE_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.rendering.composition-2d.scene-command.LightRoute2d";
pub const LAYERED_IMAGE_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.gfx.layered-image-2d.scene-command.LayeredImage2d";
pub const TILEMAP_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.gfx.tilemap-2d.scene-command.TileMap2d";
pub const SPRITE_2D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.gfx.sprite-2d.scene-command.Sprite2D";
pub const TEXT_2D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.gfx.text-2d.scene-command.Text2D";
pub const VECTOR_SHAPE_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.gfx.vector-2d.scene-command.VectorShape2D";
pub const BEACON_LIGHT_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.lighting.beacon-light-2d.scene-command.BeaconLight2d";
pub const GLOBAL_LIGHT_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.lighting.light-2d.scene-command.GlobalLight2d";
pub const LIGHTMAP_2D_SOURCE_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.lighting.light-2d.scene-command.LightMap2dSource";
pub const LIGHT_GROUP_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.lighting.light-2d.scene-command.LightGroup2d";
pub const CAMERA_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.camera-core.scene-command.Camera2d";
pub const DEPTH_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.focus-depth.scene-command.DepthMap2d";
pub const DEPTH_AUX_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.focus-depth.scene-command.DepthAuxMap2d";

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
pub struct Visual2dSpatialPluginSceneCommandPayload(pub DepthSpace2dSceneCommand);

impl PluginSceneCommandPayload for Visual2dSpatialPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        VISUAL2D_SPATIAL_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<DepthSpace2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn visual2d_spatial_plugin_scene_command(
    depth_space: DepthSpace2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Visual2dSpatialPluginSceneCommandPayload(
        depth_space,
    )))
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

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImage2dPluginSceneCommandPayload(pub LayeredImage2dSceneCommand);

impl PluginSceneCommandPayload for LayeredImage2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        LAYERED_IMAGE_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<LayeredImage2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        let mut dependencies = vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.asset.clone(),
            "layered-images",
            "layered-image-2d",
        )];
        push_visual_map_dependencies(
            &mut dependencies,
            &command.source_mod,
            command.visual_maps.as_ref(),
        );
        for override_ in &command.layer_overrides {
            push_visual_map_dependencies(
                &mut dependencies,
                &command.source_mod,
                override_.visual_maps.as_ref(),
            );
        }
        dependencies
    }
}

pub fn layered_image_2d_plugin_scene_command(
    command: LayeredImage2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(LayeredImage2dPluginSceneCommandPayload(command)))
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
pub struct DepthMap2dPluginSceneCommandPayload(pub DepthMap2dSceneCommand);

impl PluginSceneCommandPayload for DepthMap2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        DEPTH_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<DepthMap2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.asset.clone(),
            "depth-maps",
            "depth-map-2d",
        )]
    }
}

pub fn depth_map_2d_plugin_scene_command(command: DepthMap2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(DepthMap2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct DepthAuxMap2dPluginSceneCommandPayload(pub DepthAuxMap2dSceneCommand);

impl PluginSceneCommandPayload for DepthAuxMap2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        DEPTH_AUX_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<DepthAuxMap2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        let mut dependencies = vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.asset.clone(),
            "depth-maps",
            "depth-aux-map-2d",
        )];
        if let Some(surface_asset) = &command.surface_asset {
            dependencies.push(SceneAssetDependency::new(
                command.source_mod.clone(),
                surface_asset.clone(),
                "visual-maps",
                "image-2d",
            ));
        }
        dependencies
    }
}

pub fn depth_aux_map_2d_plugin_scene_command(
    command: DepthAuxMap2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(DepthAuxMap2dPluginSceneCommandPayload(command)))
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
pub struct GlobalLight2dPluginSceneCommandPayload(pub GlobalLight2dSceneCommand);

impl PluginSceneCommandPayload for GlobalLight2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        GLOBAL_LIGHT_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<GlobalLight2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn global_light_2d_plugin_scene_command(
    command: GlobalLight2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(GlobalLight2dPluginSceneCommandPayload(command)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dSourcePluginSceneCommandPayload(pub LightMap2dSourceSceneCommand);

impl PluginSceneCommandPayload for LightMap2dSourcePluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        LIGHTMAP_2D_SOURCE_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<LightMap2dSourceSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn lightmap_2d_source_plugin_scene_command(
    command: LightMap2dSourceSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(LightMap2dSourcePluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct Camera2dPluginSceneCommandPayload(pub Camera2dSceneCommand);

impl PluginSceneCommandPayload for Camera2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        CAMERA_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Camera2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        let mut dependencies = Vec::new();
        push_camera_profile_dependency(
            &mut dependencies,
            &command.source_mod,
            &command.lens.profile,
            "camera/lens",
            "camera-lens-profile-2d",
        );
        if let Some(rain_profile) = command.lens_surface.rain_profile.as_deref() {
            push_camera_profile_dependency(
                &mut dependencies,
                &command.source_mod,
                rain_profile,
                "camera/rain",
                "camera-rain-glass-profile-2d",
            );
        }
        push_camera_profile_dependency(
            &mut dependencies,
            &command.source_mod,
            &command.film.profile,
            "camera/film",
            "camera-film-stock-2d",
        );
        push_camera_profile_dependency(
            &mut dependencies,
            &command.source_mod,
            &command.look.profile,
            "camera/look",
            "camera-look-profile-2d",
        );
        dependencies
    }
}

pub fn camera_2d_plugin_scene_command(command: Camera2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Camera2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLayer2dPluginSceneCommandPayload(pub RenderLayer2dSceneCommand);

impl PluginSceneCommandPayload for RenderLayer2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        RENDER_LAYER_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<RenderLayer2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn render_layer_2d_plugin_scene_command(
    command: RenderLayer2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(RenderLayer2dPluginSceneCommandPayload(command)))
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
pub struct LightRoute2dPluginSceneCommandPayload(pub LightRoute2dSceneCommand);

impl PluginSceneCommandPayload for LightRoute2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        LIGHT_ROUTE_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<LightRoute2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn light_route_2d_plugin_scene_command(
    command: LightRoute2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(LightRoute2dPluginSceneCommandPayload(command)))
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
pub struct LightGroup2dPluginSceneCommandPayload(pub LightGroup2dSceneCommand);

impl PluginSceneCommandPayload for LightGroup2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        LIGHT_GROUP_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<LightGroup2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn light_group_2d_plugin_scene_command(
    command: LightGroup2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(LightGroup2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dPluginSceneCommandPayload(pub Sprite2dSceneCommand);

impl PluginSceneCommandPayload for Sprite2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        SPRITE_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Sprite2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        let mut dependencies = vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.texture.clone(),
            "spritesheets",
            "sprite-sheet-2d",
        )];
        push_visual_map_dependencies(
            &mut dependencies,
            &command.source_mod,
            command.visual_maps.as_ref(),
        );
        dependencies
    }
}

pub fn sprite_2d_plugin_scene_command(command: Sprite2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Sprite2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2dPluginSceneCommandPayload(pub TileMap2dSceneCommand);

impl PluginSceneCommandPayload for TileMap2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        TILEMAP_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<TileMap2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        let mut dependencies = vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.tileset.clone(),
            "tilemaps",
            "tilemap-2d",
        )];
        if let Some(ruleset) = &command.ruleset {
            dependencies.push(SceneAssetDependency::new(
                command.source_mod.clone(),
                ruleset.clone(),
                "tilemaps",
                "tile-ruleset-2d",
            ));
        }
        if let Some(sprite_sheet) = command
            .tileset
            .as_str()
            .split_once("/tilesets/")
            .map(|(sprite_sheet, _)| AssetKey::new(sprite_sheet.to_owned()))
        {
            dependencies.push(SceneAssetDependency::new(
                command.source_mod.clone(),
                sprite_sheet,
                "spritesheets",
                "sprite-sheet-2d",
            ));
        }
        dependencies
    }
}

pub fn tilemap_2d_plugin_scene_command(command: TileMap2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(TileMap2dPluginSceneCommandPayload(command)))
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
    pub post_fx_host_id: Option<amigo_render_api::PostFxHost2dId>,
    pub z_index: f32,
    pub material: Option<Material2dSceneCommand>,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text2dPluginSceneCommandPayload(pub Text2dSceneCommand);

impl PluginSceneCommandPayload for Text2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        TEXT_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Text2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<SceneAssetDependency> {
        let command = &self.0;
        vec![SceneAssetDependency::new(
            command.source_mod.clone(),
            command.font.clone(),
            "fonts",
            "font-2d",
        )]
    }
}

fn push_visual_map_dependencies(
    dependencies: &mut Vec<SceneAssetDependency>,
    source_mod: &str,
    maps: Option<&VisualMaps2dSceneCommand>,
) {
    let Some(maps) = maps else {
        return;
    };
    for asset in [
        maps.normal.as_ref(),
        maps.wetness.as_ref(),
        maps.emissive.as_ref(),
        maps.highlight.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        dependencies.push(SceneAssetDependency::new(
            source_mod.to_owned(),
            asset.clone(),
            "visual-maps",
            "image-2d",
        ));
    }
}

fn push_camera_profile_dependency(
    dependencies: &mut Vec<SceneAssetDependency>,
    source_mod: &str,
    profile: &str,
    domain_scope: &'static str,
    domain_tag: &'static str,
) {
    let profile = profile.trim();
    if profile.is_empty() || !profile.contains('/') {
        return;
    }

    let (profile_source_mod, key) = if profile.split('/').nth(1).is_some_and(|part| part == "camera")
    {
        (
            profile.split('/').next().unwrap_or(source_mod).to_owned(),
            AssetKey::new(profile.to_owned()),
        )
    } else {
        (
            source_mod.to_owned(),
            AssetKey::new(format!("{source_mod}/{profile}")),
        )
    };

    dependencies.push(SceneAssetDependency::new(
        profile_source_mod,
        key,
        domain_scope,
        domain_tag,
    ));
}

pub fn text_2d_plugin_scene_command(command: Text2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Text2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dPluginSceneCommandPayload(pub VectorShape2dSceneCommand);

impl PluginSceneCommandPayload for VectorShape2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        VECTOR_SHAPE_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<VectorShape2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn vector_shape_2d_plugin_scene_command(
    command: VectorShape2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(VectorShape2dPluginSceneCommandPayload(command)))
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

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconLight2dPluginSceneCommandPayload(pub BeaconLight2dSceneCommand);

impl PluginSceneCommandPayload for BeaconLight2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        BEACON_LIGHT_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<BeaconLight2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn beacon_light_2d_plugin_scene_command(
    command: BeaconLight2dSceneCommand,
) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(BeaconLight2dPluginSceneCommandPayload(command)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dependency_tuples(
        dependencies: Vec<SceneAssetDependency>,
    ) -> Vec<(String, String, &'static str, &'static str)> {
        dependencies
            .into_iter()
            .map(|dependency| {
                (
                    dependency.source_mod,
                    dependency.key.as_str().to_owned(),
                    dependency.domain_scope,
                    dependency.domain_tag,
                )
            })
            .collect()
    }

    #[test]
    fn sprite_asset_dependencies_include_texture_and_visual_maps() {
        let mut command = Sprite2dSceneCommand::new(
            "test-mod",
            "hero",
            AssetKey::new("test-mod/spritesheets/hero"),
            Vec2::new(16.0, 16.0),
        );
        command.visual_maps = Some(VisualMaps2dSceneCommand {
            normal: Some(AssetKey::new("test-mod/visual-maps/hero-normal")),
            wetness: None,
            emissive: Some(AssetKey::new("test-mod/visual-maps/hero-emissive")),
            highlight: None,
            roughness: None,
        });

        assert_eq!(
            dependency_tuples(Sprite2dPluginSceneCommandPayload(command).asset_dependencies()),
            vec![
                (
                    "test-mod".to_owned(),
                    "test-mod/spritesheets/hero".to_owned(),
                    "spritesheets",
                    "sprite-sheet-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/visual-maps/hero-normal".to_owned(),
                    "visual-maps",
                    "image-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/visual-maps/hero-emissive".to_owned(),
                    "visual-maps",
                    "image-2d",
                ),
            ]
        );
    }

    #[test]
    fn tilemap_asset_dependencies_include_ruleset_and_backing_spritesheet() {
        let mut command = TileMap2dSceneCommand::new(
            "test-mod",
            "level",
            AssetKey::new("test-mod/spritesheets/city/tilesets/base"),
            Vec2::new(8.0, 8.0),
            vec!["##".to_owned()],
        );
        command.ruleset = Some(AssetKey::new("test-mod/spritesheets/city/rulesets/base"));

        assert_eq!(
            dependency_tuples(TileMap2dPluginSceneCommandPayload(command).asset_dependencies()),
            vec![
                (
                    "test-mod".to_owned(),
                    "test-mod/spritesheets/city/tilesets/base".to_owned(),
                    "tilemaps",
                    "tilemap-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/spritesheets/city/rulesets/base".to_owned(),
                    "tilemaps",
                    "tile-ruleset-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/spritesheets/city".to_owned(),
                    "spritesheets",
                    "sprite-sheet-2d",
                ),
            ]
        );
    }

    #[test]
    fn camera_asset_dependencies_normalize_profile_keys() {
        let command = Camera2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "camera".to_owned(),
            camera_id: "main".to_owned(),
            mode: CameraExposureMode2dSceneCommand::Manual,
            render_contributions: RenderContributions2dSceneCommand::default(),
            exposure: CameraExposure2dSceneCommand {
                iso: 100.0,
                compensation: 0.0,
                white_balance: 6500.0,
                nd_stops: 0.0,
                auto: CameraAutoExposure2dSceneCommand {
                    target_luma: 0.5,
                    adaptation_speed: 1.0,
                    min_iso: 100.0,
                    max_iso: 800.0,
                },
            },
            shutter: CameraShutter2dSceneCommand {
                enabled: false,
                speed_s: None,
                fps: 60.0,
                angle: 180.0,
                opacity: 1.0,
                history_mix: 0.0,
                history_mix_2: 0.0,
                edge_rejection: 0.0,
                luma_threshold: 0.0,
                frame_hold: false,
            },
            lens: CameraLens2dSceneCommand {
                profile: "camera/lens/dirty".to_owned(),
                intensity: 1.0,
                aberration_px: None,
                distortion: None,
                vignette: None,
                edge_softness_px: None,
                glare_strength: None,
                dirt: None,
                focal_length_mm: None,
                lens_bloom: None,
                flare_ghosts: None,
                anamorphic_squeeze: None,
                coma: None,
                cat_eye_bokeh: None,
                focus_breathing: None,
            },
            lens_surface: CameraLensSurface2dSceneCommand {
                rain_profile: Some("shared/camera/rain/streaks".to_owned()),
            },
            film: CameraFilm2dSceneCommand {
                profile: "camera/film/print".to_owned(),
                intensity: 1.0,
                seed: 7,
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
            },
            look: CameraLook2dSceneCommand {
                profile: "camera/look/neon".to_owned(),
                intensity: 1.0,
            },
            aperture: CameraAperture2dSceneCommand {
                enabled: false,
                f_stop: 4.0,
                focus_distance_m: 2.0,
                focus: CameraFocus2dSceneCommand::None,
                depth_of_field: CameraDepthOfField2dSceneCommand {
                    depth_map: None,
                    affected_layers: Vec::new(),
                    max_blur_px: 0.0,
                    depth_contrast: 1.0,
                    focus_width: 0.1,
                    foreground_blur_boost: 1.0,
                    background_blur_boost: 1.0,
                    edge_aware: false,
                    invert_depth: false,
                    debug_view: "none".to_owned(),
                    aperture_blades: 6,
                    aperture_roundness: 1.0,
                    aperture_rotation_degrees: 0.0,
                    sample_count: 8,
                    highlight_threshold: 1.0,
                    highlight_knee: 0.5,
                    highlight_gain: 1.0,
                    highlight_saturation: 1.0,
                },
            },
        };

        assert_eq!(
            dependency_tuples(Camera2dPluginSceneCommandPayload(command).asset_dependencies()),
            vec![
                (
                    "test-mod".to_owned(),
                    "test-mod/camera/lens/dirty".to_owned(),
                    "camera/lens",
                    "camera-lens-profile-2d",
                ),
                (
                    "shared".to_owned(),
                    "shared/camera/rain/streaks".to_owned(),
                    "camera/rain",
                    "camera-rain-glass-profile-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/camera/film/print".to_owned(),
                    "camera/film",
                    "camera-film-stock-2d",
                ),
                (
                    "test-mod".to_owned(),
                    "test-mod/camera/look/neon".to_owned(),
                    "camera/look",
                    "camera-look-profile-2d",
                ),
            ]
        );
    }
}
