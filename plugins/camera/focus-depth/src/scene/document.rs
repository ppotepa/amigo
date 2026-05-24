use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    DepthAuxMap2dChannelsDocument, LayeredImageViewportFit2dDocument, SceneComponentDocument,
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError,
    SceneDocumentResult, SceneVec2Document,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FocusDepthResponse2dDocument {
    pub enabled: bool,
    pub strength: f32,
    pub focus_width_m: f32,
    pub max_blur_px: f32,
}

impl Default for FocusDepthResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: 0.0,
            focus_width_m: 1.0,
            max_blur_px: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DepthMap2dDocument {
    pub id: String,
    pub asset: String,
    pub size: SceneVec2Document,
    #[serde(default)]
    pub viewport_fit: LayeredImageViewportFit2dDocument,
    #[serde(default = "default_true")]
    pub white_is_near: bool,
    #[serde(default)]
    pub z_index: f32,
}

impl DepthMap2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::DepthMap2d {
                id,
                asset,
                size,
                viewport_fit,
                white_is_near,
                z_index,
            } => Some(Self {
                id: id.clone(),
                asset: asset.clone(),
                size: *size,
                viewport_fit: *viewport_fit,
                white_is_near: *white_is_near,
                z_index: *z_index,
            }),
            _ => None,
        }
    }
}

impl SceneComponentPayload for DepthMap2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthMap2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DepthAuxMap2dDocument {
    pub id: String,
    pub asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_asset: Option<String>,
    pub size: SceneVec2Document,
    #[serde(default)]
    pub viewport_fit: LayeredImageViewportFit2dDocument,
    #[serde(default)]
    pub channels: DepthAuxMap2dChannelsDocument,
    #[serde(default)]
    pub z_index: f32,
}

impl DepthAuxMap2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::DepthAuxMap2d {
                id,
                asset,
                surface_asset,
                size,
                viewport_fit,
                channels,
                z_index,
            } => Some(Self {
                id: id.clone(),
                asset: asset.clone(),
                surface_asset: surface_asset.clone(),
                size: *size,
                viewport_fit: *viewport_fit,
                channels: channels.clone(),
                z_index: *z_index,
            }),
            _ => None,
        }
    }
}

impl SceneComponentPayload for DepthAuxMap2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthAuxMap2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_depth_map_2d_plugin_payload(payload: &Value) -> SceneDocumentResult<DepthMap2dDocument> {
    serde_yaml::from_value::<DepthMap2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn parse_depth_aux_map_2d_plugin_payload(
    payload: &Value,
) -> SceneDocumentResult<DepthAuxMap2dDocument> {
    serde_yaml::from_value::<DepthAuxMap2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct DepthMap2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for DepthMap2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthMap2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["DepthMap2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<DepthMap2dDocument>(Value::Mapping(payload))?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_depth_map_2d_plugin_payload(payload)?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepthAuxMap2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for DepthAuxMap2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthAuxMap2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["DepthAuxMap2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<DepthAuxMap2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_depth_aux_map_2d_plugin_payload(payload)?))
    }
}

fn default_true() -> bool {
    true
}
