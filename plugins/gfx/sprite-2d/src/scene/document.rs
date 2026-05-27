use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    Material2dDocument, RenderContributionsDocument, SceneComponentDocument, SceneComponentPayload,
    SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
    SceneSpriteAnimationDocument, SceneSpriteSheetDocument, SceneVec2Document,
    VisualMaps2dDocument,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sprite2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub texture: String,
    pub size: SceneVec2Document,
    #[serde(default)]
    pub sheet: Option<SceneSpriteSheetDocument>,
    #[serde(default)]
    pub animation: Option<SceneSpriteAnimationDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_maps: Option<VisualMaps2dDocument>,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material2dDocument>,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_visible")]
    pub visible: bool,
}

impl Sprite2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.sprite-2d.Sprite2D"
                || component_type == "Sprite2D" =>
            {
                parse_sprite_2d_plugin_payload(payload).ok()
            }
            _ => None,
        }
    }
}

impl SceneComponentPayload for Sprite2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.sprite-2d.Sprite2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_sprite_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Sprite2dDocument> {
    serde_yaml::from_value::<Sprite2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sprite2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for Sprite2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.sprite-2d.Sprite2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Sprite2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Sprite2dDocument>(Value::Mapping(
            payload,
        ))?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_sprite_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}

fn default_opacity() -> f32 {
    1.0
}

fn default_visible() -> bool {
    true
}
