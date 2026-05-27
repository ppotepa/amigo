use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    SceneComponentDocument, SceneComponentPayload, SceneComponentSchemaProvider,
    SceneDocumentError, SceneDocumentResult,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GlobalLight2dDocument {
    pub id: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_intensity")]
    pub intensity: f32,
}

impl GlobalLight2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.lighting.light-2d.GlobalLight2D"
                || component_type == "GlobalLight2D" =>
            {
                parse_global_light_2d_plugin_payload(payload).ok()
            }
            _ => None,
        }
    }
}

impl SceneComponentPayload for GlobalLight2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.lighting.light-2d.GlobalLight2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_global_light_2d_plugin_payload(
    payload: &Value,
) -> SceneDocumentResult<GlobalLight2dDocument> {
    serde_yaml::from_value::<GlobalLight2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalLight2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for GlobalLight2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.lighting.light-2d.GlobalLight2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["GlobalLight2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<GlobalLight2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_global_light_2d_plugin_payload(payload)?))
    }
}

fn default_color() -> String {
    "#FFFFFFFF".to_owned()
}

fn default_intensity() -> f32 {
    1.0
}
