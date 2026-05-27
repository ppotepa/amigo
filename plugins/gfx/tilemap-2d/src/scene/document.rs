use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    SceneComponentDocument, SceneComponentPayload, SceneComponentSchemaProvider,
    SceneDocumentError, SceneDocumentResult, SceneVec2Document, TileMap2dEditorDocument,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tilemap2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub tileset: String,
    #[serde(default)]
    pub ruleset: Option<String>,
    pub tile_size: SceneVec2Document,
    #[serde(default)]
    pub editor: Option<TileMap2dEditorDocument>,
    #[serde(default)]
    pub grid: Vec<String>,
    #[serde(default)]
    pub depth_fill_rows: usize,
    #[serde(default)]
    pub z_index: f32,
}

impl Default for Tilemap2dDocument {
    fn default() -> Self {
        Self {
            entity_name: String::new(),
            render_layer: default_render_layer(),
            tileset: String::new(),
            ruleset: None,
            tile_size: SceneVec2Document { x: 0.0, y: 0.0 },
            editor: None,
            grid: Vec::new(),
            depth_fill_rows: 0,
            z_index: 0.0,
        }
    }
}

impl Tilemap2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.tilemap-2d.TileMap2D"
                || component_type == "TileMap2D" =>
            {
                parse_tilemap_2d_plugin_payload(payload).ok()
            }
            _ => None,
        }
    }
}

impl SceneComponentPayload for Tilemap2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.tilemap-2d.TileMap2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_tilemap_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Tilemap2dDocument> {
    serde_yaml::from_value::<Tilemap2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TileMap2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for TileMap2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.tilemap-2d.TileMap2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["TileMap2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Tilemap2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_tilemap_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}
