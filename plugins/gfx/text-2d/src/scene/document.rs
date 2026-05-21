use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_material_api::Material2dDocument;
use amigo_scene::{
    RenderContributionsDocument, SceneComponentDocument, SceneComponentSchemaProvider,
    SceneDocumentError, SceneDocumentResult, SceneVec2Document, Text2dStyleDocument,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Text2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub content: String,
    pub font: String,
    pub bounds: SceneVec2Document,
    #[serde(default)]
    pub style: Text2dStyleDocument,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material2dDocument>,
}

impl Text2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Text2d {
                render_layer,
                content,
                font,
                bounds,
                style,
                render_contributions,
                z_index,
                material,
                post_fx: _,
            } => Some(Self {
                entity_name: String::new(),
                render_layer: render_layer.clone(),
                content: content.clone(),
                font: font.clone(),
                bounds: *bounds,
                style: style.clone(),
                render_contributions: render_contributions.clone(),
                z_index: *z_index,
                material: *material,
            }),
            _ => None,
        }
    }
}

pub fn parse_text_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Text2dDocument> {
    serde_yaml::from_value::<Text2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for Text2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.text-2d.Text2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Text2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Text2dDocument>(Value::Mapping(payload))?)
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}
