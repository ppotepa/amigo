use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetKey(String);

impl AssetKey {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSourceKind {
    Engine,
    Mod(String),
    FileSystemRoot(String),
    Generated,
}

impl AssetSourceKind {
    pub fn label(&self) -> String {
        match self {
            Self::Engine => "engine".to_owned(),
            Self::Mod(mod_id) => format!("mod:{mod_id}"),
            Self::FileSystemRoot(root) => format!("fs:{root}"),
            Self::Generated => "generated".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AssetLoadPriority {
    Background,
    #[default]
    Interactive,
    Immediate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetManifest {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub tags: Vec<String>,
}

impl AssetManifest {
    pub fn engine(key: AssetKey) -> Self {
        Self {
            key,
            source: AssetSourceKind::Engine,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetLoadRequest {
    pub key: AssetKey,
    pub priority: AssetLoadPriority,
}

impl AssetLoadRequest {
    pub fn new(key: AssetKey, priority: AssetLoadPriority) -> Self {
        Self { key, priority }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetEvent {
    ManifestRegistered(AssetKey),
    LoadRequested(AssetLoadRequest),
    ReloadRequested(AssetLoadRequest),
    LoadCompleted(AssetKey),
    LoadFailed { key: AssetKey, reason: String },
    Prepared(AssetKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedAsset {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub resolved_path: PathBuf,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedAsset {
    pub key: AssetKey,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedAssetKind {
    Image2d,
    Sprite2d,
    SpriteSheet2d,
    TileSet2d,
    TileRuleSet2d,
    Font2d,
    Font3d,
    Mesh3d,
    Material3d,
    GeneratedAudio,
    Unknown(String),
}

impl PreparedAssetKind {
    pub fn from_placeholder_kind(value: &str) -> Self {
        match value {
            "image-2d" => Self::Image2d,
            "sprite-2d" => Self::Sprite2d,
            "sprite-sheet-2d" | "spritesheet-2d" => Self::SpriteSheet2d,
            "tileset-2d" => Self::TileSet2d,
            "tile-ruleset-2d" => Self::TileRuleSet2d,
            "font-2d" => Self::Font2d,
            "font-3d" => Self::Font3d,
            "mesh-3d" => Self::Mesh3d,
            "material-3d" => Self::Material3d,
            "generated-audio" => Self::GeneratedAudio,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Image2d => "image-2d",
            Self::Sprite2d => "sprite-2d",
            Self::SpriteSheet2d => "sprite-sheet-2d",
            Self::TileSet2d => "tileset-2d",
            Self::TileRuleSet2d => "tile-ruleset-2d",
            Self::Font2d => "font-2d",
            Self::Font3d => "font-3d",
            Self::Mesh3d => "mesh-3d",
            Self::Material3d => "material-3d",
            Self::GeneratedAudio => "generated-audio",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAsset {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub resolved_path: PathBuf,
    pub byte_len: u64,
    pub kind: PreparedAssetKind,
    pub label: Option<String>,
    pub format: Option<String>,
    pub metadata: BTreeMap<String, String>,
}
