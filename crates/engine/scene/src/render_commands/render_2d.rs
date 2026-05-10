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

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredImageLayerOverrideSceneCommand {
    pub id: String,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<LayeredImageBlendMode2dSceneCommand>,
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
    pub z_index: f32,
    pub transform: Transform2,
    pub layer_overrides: Vec<LayeredImageLayerOverrideSceneCommand>,
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
pub struct RenderLayer2dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub order: f32,
    pub visible: bool,
    pub opacity: f32,
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
    pub z_index: f32,
    pub transform: Transform2,
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
            z_index: 0.0,
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
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}
