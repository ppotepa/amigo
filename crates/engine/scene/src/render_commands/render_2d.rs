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
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
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
    pub content: String,
    pub font: AssetKey,
    pub bounds: Vec2,
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
            content: content.into(),
            font,
            bounds,
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
            kind,
            style,
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}
