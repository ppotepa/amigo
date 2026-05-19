use super::{default_one, default_white_color};
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraOpticalResponse2dDocument {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub intensity: f32,
    #[serde(default)]
    pub bloom: f32,
    #[serde(default)]
    pub glare: f32,
    #[serde(default)]
    pub ghosting: f32,
    #[serde(default)]
    pub streaks: f32,
    #[serde(default)]
    pub chromatic_smear: f32,
    #[serde(default)]
    pub dirt_response: f32,
    #[serde(default)]
    pub halation: f32,
    #[serde(default)]
    pub threshold: f32,
}

impl Default for CameraOpticalResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.0,
            bloom: 0.0,
            glare: 0.0,
            ghosting: 0.0,
            streaks: 0.0,
            chromatic_smear: 0.0,
            dirt_response: 0.0,
            halation: 0.0,
            threshold: 0.0,
        }
    }
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
