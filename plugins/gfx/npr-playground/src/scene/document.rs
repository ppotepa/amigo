use amigo_scene::{
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::any::Any;

pub const NPR_SETTINGS_COMPONENT_TYPE: &str = "amigo.gfx.npr-playground.NprSettings";

/// Scene-relative authored sidecar; client selection is declared by playgrounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprPlaygroundSceneDocument {
    pub profile: String,
}

impl amigo_scene::SceneComponentPayload for NprPlaygroundSceneDocument {
    fn component_type(&self) -> &'static str {
        NPR_SETTINGS_COMPONENT_TYPE
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NprPlaygroundSceneSchemaProvider;

impl SceneComponentSchemaProvider for NprPlaygroundSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        NPR_SETTINGS_COMPONENT_TYPE
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<NprPlaygroundSceneDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        serde_yaml::from_value::<NprPlaygroundSceneDocument>(payload.clone())
            .map(|document| Box::new(document) as Box<dyn SceneComponentPayload>)
            .map_err(|source| SceneDocumentError::Parse { path: None, source })
    }
}
