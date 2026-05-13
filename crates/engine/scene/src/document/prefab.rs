use serde::{Deserialize, Serialize};

use super::core::SceneEntityDocument;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefabDocument {
    pub version: u32,
    pub prefab: PrefabMetadataDocument,
    pub root: String,
    #[serde(default)]
    pub entities: Vec<SceneEntityDocument>,
    #[serde(default)]
    pub exposed: Vec<PrefabExposedPropertyDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefabMetadataDocument {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefabExposedPropertyDocument {
    pub name: String,
    pub target: String,
    pub kind: PrefabExposedPropertyKindDocument,
    #[serde(default)]
    pub default: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrefabExposedPropertyKindDocument {
    String,
    Number,
    Bool,
    Vec2,
    Vec3,
    Color,
    AssetRef,
}

