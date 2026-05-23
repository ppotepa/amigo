use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_material_api::Material2dDocument;
use amigo_scene::{
    RenderContributionsDocument, SceneComponentDocument, SceneComponentSchemaProvider,
    SceneComponentPayload, SceneDocumentError, SceneDocumentResult,
    SceneVectorShapeKindComponentDocument, SceneVec2Document,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vector2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub kind: SceneVectorShapeKindComponentDocument,
    #[serde(default)]
    pub points: Vec<SceneVec2Document>,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub radius: f32,
    #[serde(default = "default_segments")]
    pub segments: u32,
    #[serde(default)]
    pub stroke_color: Option<String>,
    #[serde(default = "default_stroke_width")]
    pub stroke_width: f32,
    #[serde(default)]
    pub fill_color: Option<String>,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material2dDocument>,
    #[serde(default)]
    pub z_index: f32,
}

impl Vector2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::VectorShape2d {
                render_layer,
                kind,
                points,
                closed,
                radius,
                segments,
                stroke_color,
                stroke_width,
                fill_color,
                render_contributions,
                material,
                z_index,
                post_fx: _,
            } => Some(Self {
                entity_name: String::new(),
                render_layer: render_layer.clone(),
                kind: kind.clone(),
                points: points.clone(),
                closed: *closed,
                radius: *radius,
                segments: *segments,
                stroke_color: stroke_color.clone(),
                stroke_width: *stroke_width,
                fill_color: fill_color.clone(),
                render_contributions: render_contributions.clone(),
                material: *material,
                z_index: *z_index,
            }),
            _ => None,
        }
    }
}

impl SceneComponentPayload for Vector2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.vector-2d.VectorShape2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_vector_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<Vector2dDocument> {
    serde_yaml::from_value::<Vector2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for Vector2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.vector-2d.VectorShape2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["VectorShape2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<Vector2dDocument>(Value::Mapping(payload))?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_vector_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}

fn default_segments() -> u32 {
    32
}

fn default_stroke_width() -> f32 {
    1.0
}
