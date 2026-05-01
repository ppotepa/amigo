#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
}

impl Mesh3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        mesh_asset: AssetKey,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            mesh_asset,
            transform: Transform3::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

impl Text3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        content: impl Into<String>,
        font: AssetKey,
        size: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            content: content.into(),
            font,
            size,
            transform: Transform3::default(),
        }
    }
}

