use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{SceneDocumentError, SceneDocumentResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneDocument {
    #[serde(default = "default_scene_document_version")]
    pub version: u32,
    pub scene: SceneMetadataDocument,
    #[serde(default)]
    pub transitions: Vec<SceneTransitionDocument>,
    #[serde(default)]
    pub entities: Vec<SceneEntityDocument>,
}

impl SceneDocument {
    pub fn entity_names(&self) -> Vec<String> {
        self.entities
            .iter()
            .map(SceneEntityDocument::display_name)
            .collect()
    }

    pub fn component_kind_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();

        for entity in &self.entities {
            for component in &entity.components {
                *counts.entry(component.kind().to_owned()).or_insert(0) += 1;
            }
        }

        counts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneMetadataDocument {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneTransitionDocument {
    #[serde(default)]
    pub id: String,
    pub to: String,
    pub when: SceneTransitionConditionDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneTransitionConditionDocument {
    AfterSeconds {
        seconds: f32,
    },
    ScriptEvent {
        topic: String,
        #[serde(default)]
        payload: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneEntityDocument {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub transform2: Option<SceneTransform2Document>,
    #[serde(default)]
    pub transform3: Option<SceneTransform3Document>,
    #[serde(default)]
    pub components: Vec<SceneComponentDocument>,
}

impl SceneEntityDocument {
    pub fn display_name(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SceneComponentDocument {
    #[serde(rename = "Camera2D")]
    Camera2d,
    #[serde(rename = "Camera3D")]
    Camera3d,
    #[serde(rename = "Light3D")]
    Light3d {
        #[serde(default)]
        kind: String,
    },
    #[serde(rename = "Sprite2D")]
    Sprite2d {
        texture: String,
        size: SceneVec2Document,
        #[serde(default)]
        sheet: Option<SceneSpriteSheetDocument>,
    },
    #[serde(rename = "Text2D")]
    Text2d {
        content: String,
        font: String,
        bounds: SceneVec2Document,
    },
    #[serde(rename = "Mesh3D")]
    Mesh3d { mesh: String },
    #[serde(rename = "Material3D")]
    Material3d {
        label: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        albedo: Option<String>,
    },
    #[serde(rename = "Text3D")]
    Text3d {
        content: String,
        font: String,
        size: f32,
    },
}

impl SceneComponentDocument {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Camera2d => "Camera2D",
            Self::Camera3d => "Camera3D",
            Self::Light3d { .. } => "Light3D",
            Self::Sprite2d { .. } => "Sprite2D",
            Self::Text2d { .. } => "Text2D",
            Self::Mesh3d { .. } => "Mesh3D",
            Self::Material3d { .. } => "Material3D",
            Self::Text3d { .. } => "Text3D",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneSpriteSheetDocument {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: SceneVec2Document,
    #[serde(default = "default_sprite_sheet_fps")]
    pub fps: f32,
    #[serde(default = "default_sprite_sheet_looping")]
    pub looping: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneVec2Document {
    pub x: f32,
    pub y: f32,
}

impl SceneVec2Document {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneVec3Document {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SceneVec3Document {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneTransform2Document {
    #[serde(default = "default_vec2_zero")]
    pub translation: SceneVec2Document,
    #[serde(default)]
    pub rotation_radians: f32,
    #[serde(default = "default_vec2_one")]
    pub scale: SceneVec2Document,
}

impl Default for SceneTransform2Document {
    fn default() -> Self {
        Self {
            translation: default_vec2_zero(),
            rotation_radians: 0.0,
            scale: default_vec2_one(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneTransform3Document {
    #[serde(default = "default_vec3_zero")]
    pub translation: SceneVec3Document,
    #[serde(default = "default_vec3_zero")]
    pub rotation_euler: SceneVec3Document,
    #[serde(default = "default_vec3_one")]
    pub scale: SceneVec3Document,
}

impl Default for SceneTransform3Document {
    fn default() -> Self {
        Self {
            translation: default_vec3_zero(),
            rotation_euler: default_vec3_zero(),
            scale: default_vec3_one(),
        }
    }
}

pub fn load_scene_document_from_str(source: &str) -> SceneDocumentResult<SceneDocument> {
    serde_yaml::from_str::<SceneDocument>(source)
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn load_scene_document_from_path(path: impl AsRef<Path>) -> SceneDocumentResult<SceneDocument> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| SceneDocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str::<SceneDocument>(&raw).map_err(|source| SceneDocumentError::Parse {
        path: Some(path.to_path_buf()),
        source,
    })
}

pub fn scene_document_path(
    mod_root: impl AsRef<Path>,
    relative_document_path: impl AsRef<Path>,
) -> PathBuf {
    mod_root.as_ref().join(relative_document_path.as_ref())
}

fn default_scene_document_version() -> u32 {
    1
}

fn default_vec2_zero() -> SceneVec2Document {
    SceneVec2Document::ZERO
}

fn default_vec2_one() -> SceneVec2Document {
    SceneVec2Document::ONE
}

fn default_vec3_zero() -> SceneVec3Document {
    SceneVec3Document::ZERO
}

fn default_vec3_one() -> SceneVec3Document {
    SceneVec3Document::ONE
}

fn default_sprite_sheet_fps() -> f32 {
    8.0
}

fn default_sprite_sheet_looping() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        SceneComponentDocument, load_scene_document_from_path, load_scene_document_from_str,
    };

    #[test]
    fn parses_scene_document_from_yaml() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        assert_eq!(document.scene.id, "sprite-lab");
        assert_eq!(document.entities.len(), 2);
        assert_eq!(document.entity_names()[1], "playground-2d-sprite");
        assert_eq!(
            document.component_kind_counts().get("Sprite2D"),
            Some(&1usize)
        );
        assert!(matches!(
            document.entities[1].components[0],
            SceneComponentDocument::Sprite2d { .. }
        ));
    }

    #[test]
    fn parses_playground_scene_documents_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let sprite_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/sprite-lab/scene.yml"),
        )
        .expect("sprite lab scene should parse");
        let material_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
        )
        .expect("material lab scene should parse");

        assert_eq!(sprite_doc.scene.id, "sprite-lab");
        assert_eq!(material_doc.scene.id, "material-lab");
        assert!(sprite_doc.component_kind_counts().contains_key("Sprite2D"));
        assert!(
            material_doc
                .component_kind_counts()
                .contains_key("Material3D")
        );
    }

    #[test]
    fn parses_playground_2d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/hello-world-spritesheet/scene.yml"),
        )
        .expect("playground 2d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-spritesheet");
        assert_eq!(document.transitions.len(), 1);
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(document.component_kind_counts().contains_key("Text2D"));
    }

    #[test]
    fn parses_playground_3d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/hello-world-cube/scene.yml"),
        )
        .expect("playground 3d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-cube");
        assert!(document.component_kind_counts().contains_key("Mesh3D"));
        assert!(document.component_kind_counts().contains_key("Material3D"));
        assert!(document.component_kind_counts().contains_key("Text3D"));
    }
}
