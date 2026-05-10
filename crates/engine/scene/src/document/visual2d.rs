use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}

fn default_white_color() -> String {
    "#ffffff".to_owned()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneVisual2dDocument {
    #[serde(default)]
    pub render_layers: Vec<RenderLayer2dDocument>,
    #[serde(default)]
    pub light_groups: Vec<LightGroup2dDocument>,
    #[serde(default)]
    pub light_routes: Vec<LightRoute2dDocument>,
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
}

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
