use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneSchedulingDocument {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub max_workers: Option<usize>,
    #[serde(default)]
    pub allow_frame_latency: Option<bool>,
    #[serde(default)]
    pub overrides: Vec<SceneSchedulingOverrideDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneSchedulingOverrideDocument {
    pub target: String,
    #[serde(default)]
    pub lane: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub parallelism: Option<String>,
    #[serde(default)]
    pub allow_frame_latency: Option<bool>,
    #[serde(default)]
    pub quality_scale: Option<f32>,
    #[serde(default)]
    pub budget_ms: Option<f32>,
}

