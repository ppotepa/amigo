use amigo_2d_post_fx::PostFx2dStack;
use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageBlendMode2d {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

impl LayeredImageBlendMode2d {
    pub fn parse(value: &str) -> Self {
        Self::parse_strict(value).unwrap_or(Self::Alpha)
    }

    pub fn parse_strict(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "alpha" => Some(Self::Alpha),
            "add" | "additive" => Some(Self::Additive),
            "screen" => Some(Self::Screen),
            "multiply" => Some(Self::Multiply),
            "lighten" => Some(Self::Lighten),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Additive => "additive",
            Self::Screen => "screen",
            Self::Multiply => "multiply",
            Self::Lighten => "lighten",
        }
    }
}

impl From<amigo_scene::LayeredImageBlendMode2dSceneCommand> for LayeredImageBlendMode2d {
    fn from(value: amigo_scene::LayeredImageBlendMode2dSceneCommand) -> Self {
        match value {
            amigo_scene::LayeredImageBlendMode2dSceneCommand::Alpha => Self::Alpha,
            amigo_scene::LayeredImageBlendMode2dSceneCommand::Additive => Self::Additive,
            amigo_scene::LayeredImageBlendMode2dSceneCommand::Screen => Self::Screen,
            amigo_scene::LayeredImageBlendMode2dSceneCommand::Multiply => Self::Multiply,
            amigo_scene::LayeredImageBlendMode2dSceneCommand::Lighten => Self::Lighten,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayeredImageViewportFit2d {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

impl From<amigo_scene::LayeredImageViewportFit2dSceneCommand> for LayeredImageViewportFit2d {
    fn from(value: amigo_scene::LayeredImageViewportFit2dSceneCommand) -> Self {
        match value {
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed => Self::Fixed,
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Stretch => Self::Stretch,
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Contain => Self::Contain,
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Cover => Self::Cover,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayer {
    pub id: String,
    pub label: String,
    pub image: String,
    pub blend_mode: LayeredImageBlendMode2d,
    pub opacity: f32,
    pub color: Option<ColorRgba>,
    pub animation_hint: Option<String>,
    pub post_fx: Option<PostFx2dStack>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageAsset {
    pub key: AssetKey,
    pub label: Option<String>,
    pub canvas_size: Vec2,
    pub base_image: String,
    pub layers: Vec<LayeredImageLayer>,
    pub preview_image: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayerOverride {
    pub id: String,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<LayeredImageBlendMode2d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageInstance {
    pub asset: AssetKey,
    pub size: Vec2,
    pub base_opacity: f32,
    pub viewport_fit: LayeredImageViewportFit2d,
    pub layer_overrides: Vec<LayeredImageLayerOverride>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub render_layer: String,
    pub image: LayeredImageInstance,
    pub z_index: f32,
    pub transform: Transform2,
}
