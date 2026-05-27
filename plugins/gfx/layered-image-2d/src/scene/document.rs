use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::api::LayeredImage2dLayer;

use amigo_scene::{
    LayeredImageLayerOverrideDocument, LayeredImageViewportFit2dDocument, SceneComponentDocument,
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
    SceneVec2Document, VisualMaps2dDocument,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LayeredImage2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<LayeredImage2dLayer>,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    pub asset: String,
    pub size: SceneVec2Document,
    #[serde(default = "default_base_opacity")]
    pub base_opacity: f32,
    #[serde(default)]
    pub viewport_fit: LayeredImageViewportFit2dDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_maps: Option<VisualMaps2dDocument>,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default)]
    pub layer_overrides: Vec<LayeredImageLayerOverrideDocument>,
}

impl Default for LayeredImage2dDocument {
    fn default() -> Self {
        Self {
            entity_name: String::new(),
            layers: Vec::new(),
            render_layer: default_render_layer(),
            asset: String::new(),
            size: SceneVec2Document { x: 0.0, y: 0.0 },
            base_opacity: default_base_opacity(),
            viewport_fit: LayeredImageViewportFit2dDocument::default(),
            visual_maps: None,
            z_index: 0.0,
            layer_overrides: Vec::new(),
        }
    }
}

impl LayeredImage2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.layered-image-2d.LayeredImage2D"
                || component_type == "LayeredImage2D" =>
            {
                parse_layered_image_2d_plugin_payload(payload).ok()
            }
            _ => None,
        }
    }
}

impl SceneComponentPayload for LayeredImage2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_layered_image_2d_plugin_payload(
    payload: &Value,
) -> SceneDocumentResult<LayeredImage2dDocument> {
    serde_yaml::from_value::<LayeredImage2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LayeredImage2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for LayeredImage2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["LayeredImage2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<LayeredImage2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_layered_image_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "world".to_owned()
}

fn default_base_opacity() -> f32 {
    1.0
}
