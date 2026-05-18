use super::post_fx::PostFx2dDocument;
use super::{default_one, default_true};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenderDepthMode2dDocument {
    #[default]
    DepthMap,
    Distance,
    ZDepth,
    Infinity,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpticalLayerRole2dDocument {
    #[default]
    WorldSurface,
    SceneMedium,
    ForegroundMedium,
    LensSurface,
    Overlay,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderDepth2dDocument {
    #[serde(default)]
    pub mode: RenderDepthMode2dDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_m: Option<f32>,
    #[serde(default = "default_render_z_depth")]
    pub z_depth: f32,
    #[serde(default = "default_render_depth_blur_scale")]
    pub blur_scale: f32,
}

impl Default for RenderDepth2dDocument {
    fn default() -> Self {
        Self {
            mode: RenderDepthMode2dDocument::DepthMap,
            distance_m: None,
            z_depth: default_render_z_depth(),
            blur_scale: default_render_depth_blur_scale(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLayer2dDocument {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub order: f32,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default)]
    pub depth: RenderDepth2dDocument,
    #[serde(default)]
    pub optical_role: OpticalLayerRole2dDocument,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}

fn default_render_z_depth() -> f32 {
    0.5
}

fn default_render_depth_blur_scale() -> f32 {
    1.0
}
