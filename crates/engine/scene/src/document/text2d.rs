use serde::{Deserialize, Serialize};

use super::SceneVec2Document;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Text2dStyleDocument {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub align: Text2dAlignDocument,
    #[serde(default)]
    pub blend: Text2dBlendModeDocument,
    #[serde(default)]
    pub shadow: Option<Text2dShadowDocument>,
    #[serde(default)]
    pub outline: Option<Text2dOutlineDocument>,
    #[serde(default)]
    pub glow: Option<Text2dGlowDocument>,
}

impl Default for Text2dStyleDocument {
    fn default() -> Self {
        Self {
            color: None,
            opacity: None,
            font_size: None,
            align: Text2dAlignDocument::Left,
            blend: Text2dBlendModeDocument::Alpha,
            shadow: None,
            outline: None,
            glow: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Text2dAlignDocument {
    Left,
    Center,
    Right,
}

impl Default for Text2dAlignDocument {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Text2dBlendModeDocument {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

impl Default for Text2dBlendModeDocument {
    fn default() -> Self {
        Self::Alpha
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Text2dShadowDocument {
    pub color: String,
    pub offset: SceneVec2Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Text2dOutlineDocument {
    pub color: String,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Text2dGlowDocument {
    pub color: String,
    pub radius: f32,
    pub intensity: f32,
    #[serde(default = "default_text2d_glow_passes")]
    pub passes: u8,
}

fn default_text2d_glow_passes() -> u8 {
    6
}
