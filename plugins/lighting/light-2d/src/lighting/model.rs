use amigo_camera_optics_plugin::api::CameraOpticalResponse2d;
use amigo_math::ColorRgba;
use amigo_render_api::RenderContributionSet;

pub const LIGHTING_2D_CAPABILITY: &str = "lighting_2d";
pub const LIGHTING_2D_PLUGIN_LABEL: &str = "amigo-light-2d-plugin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMap2dSourceKind {
    LayeredImage2d,
}

impl From<amigo_scene::LightMap2dSourceKindSceneCommand> for LightMap2dSourceKind {
    fn from(value: amigo_scene::LightMap2dSourceKindSceneCommand) -> Self {
        match value {
            amigo_scene::LightMap2dSourceKindSceneCommand::LayeredImage2d => Self::LayeredImage2d,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dSourceRef {
    pub kind: LightMap2dSourceKind,
    pub entity_name: String,
}

impl From<&amigo_scene::LightMap2dSourceRefSceneCommand> for LightMap2dSourceRef {
    fn from(value: &amigo_scene::LightMap2dSourceRefSceneCommand) -> Self {
        Self {
            kind: value.kind.into(),
            entity_name: value.entity_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dChannel {
    pub id: String,
    pub layers: Vec<String>,
}

impl From<&amigo_scene::LightMap2dChannelSceneCommand> for LightMap2dChannel {
    fn from(value: &amigo_scene::LightMap2dChannelSceneCommand) -> Self {
        Self {
            id: value.id.clone(),
            layers: value.layers.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap2dSourceCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub source: LightMap2dSourceRef,
    pub channels: Vec<LightMap2dChannel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLight2dCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub color: ColorRgba,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightGroup2dCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub color: ColorRgba,
    pub intensity: f32,
    pub render_contributions: RenderContributionSet,
    pub camera_response: CameraOpticalResponse2d,
    pub sources: Vec<LightGroup2dSourceCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightGroup2dSourceCommand {
    pub kind: LightGroup2dSourceKind,
    pub response: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightGroup2dSourceKind {
    LightMapChannel { source: String, channel: String },
    GlobalLight { id: String },
}

impl From<amigo_scene::LightGroup2dSourceSceneCommand> for LightGroup2dSourceCommand {
    fn from(value: amigo_scene::LightGroup2dSourceSceneCommand) -> Self {
        let kind = match value.kind {
            amigo_scene::LightGroup2dSourceKindSceneCommand::LightMapChannel {
                source,
                channel,
            } => LightGroup2dSourceKind::LightMapChannel { source, channel },
            amigo_scene::LightGroup2dSourceKindSceneCommand::GlobalLight { id } => {
                LightGroup2dSourceKind::GlobalLight { id }
            }
        };
        Self {
            kind,
            response: value.response.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material2dLightingMode {
    Unlit,
    DynamicLights,
    LightMapSampled,
    LightGroupSampled,
}

impl From<amigo_scene::Material2dLightingModeSceneCommand> for Material2dLightingMode {
    fn from(value: amigo_scene::Material2dLightingModeSceneCommand) -> Self {
        match value {
            amigo_scene::Material2dLightingModeSceneCommand::Unlit => Self::Unlit,
            amigo_scene::Material2dLightingModeSceneCommand::DynamicLights => Self::DynamicLights,
            amigo_scene::Material2dLightingModeSceneCommand::LightMapSampled => {
                Self::LightMapSampled
            }
            amigo_scene::Material2dLightingModeSceneCommand::LightGroupSampled => {
                Self::LightGroupSampled
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSampleStrategy2d {
    Point,
    Line,
}

impl From<amigo_scene::LightSampleStrategy2dSceneCommand> for LightSampleStrategy2d {
    fn from(value: amigo_scene::LightSampleStrategy2dSceneCommand) -> Self {
        match value {
            amigo_scene::LightSampleStrategy2dSceneCommand::Point => Self::Point,
            amigo_scene::LightSampleStrategy2dSceneCommand::Line => Self::Line,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightReceiverDarkPolicy2d {
    Transparent,
    BaseColor,
    ShadowTint,
}

impl From<amigo_scene::LightReceiverDarkPolicy2dSceneCommand> for LightReceiverDarkPolicy2d {
    fn from(value: amigo_scene::LightReceiverDarkPolicy2dSceneCommand) -> Self {
        match value {
            amigo_scene::LightReceiverDarkPolicy2dSceneCommand::Transparent => Self::Transparent,
            amigo_scene::LightReceiverDarkPolicy2dSceneCommand::BaseColor => Self::BaseColor,
            amigo_scene::LightReceiverDarkPolicy2dSceneCommand::ShadowTint => Self::ShadowTint,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiverGlobalLight2d {
    pub id: String,
    pub response: f32,
}

impl From<&amigo_scene::LightReceiverGlobalLight2dSceneCommand> for LightReceiverGlobalLight2d {
    fn from(value: &amigo_scene::LightReceiverGlobalLight2dSceneCommand) -> Self {
        Self {
            id: value.id.clone(),
            response: value.response.max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightReceiver2dBinding {
    pub groups: Vec<String>,
    pub source: String,
    pub channel: String,
    pub sample_strategy: LightSampleStrategy2d,
    pub sample_points: u32,
    pub radius_px: f32,
    pub exposure: f32,
    pub dark_policy: LightReceiverDarkPolicy2d,
    pub global_lights: Vec<LightReceiverGlobalLight2d>,
}

impl From<&amigo_scene::LightReceiver2dBindingSceneCommand> for LightReceiver2dBinding {
    fn from(value: &amigo_scene::LightReceiver2dBindingSceneCommand) -> Self {
        Self {
            groups: value.groups.clone(),
            source: value.source.clone(),
            channel: value.channel.clone(),
            sample_strategy: value.sample_strategy.into(),
            sample_points: value.sample_points.clamp(1, 9),
            radius_px: value.radius_px.max(0.0),
            exposure: value.exposure.max(0.0),
            dark_policy: value.dark_policy.into(),
            global_lights: value.global_lights.iter().map(Into::into).collect(),
        }
    }
}
