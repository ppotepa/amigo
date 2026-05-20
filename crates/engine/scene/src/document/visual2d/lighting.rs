use super::{default_one, default_white_color};
use amigo_camera::CameraOpticalResponse2dDocument;
use crate::RenderContributionsDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightRoute2dDocument {
    pub receiver_layer: String,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightGroup2dDocument {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_white_color")]
    pub color: String,
    #[serde(default = "default_one")]
    pub intensity: f32,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default)]
    pub camera_response: CameraOpticalResponse2dDocument,
    #[serde(default)]
    pub sources: Vec<LightGroup2dSourceDocument>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LightGroup2dSourceDocument {
    LightmapChannel {
        source: String,
        channel: String,
        #[serde(default = "default_one")]
        response: f32,
    },
    GlobalLight {
        id: String,
        #[serde(default = "default_one")]
        response: f32,
    },
}
