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
    pub frame_clock: Option<SceneFrameClockDocument>,
    #[serde(default)]
    pub overrides: Vec<SceneSchedulingOverrideDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneFrameClockDocument {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub simulation_fps: Option<f32>,
    #[serde(default)]
    pub render_fps: Option<f32>,
    #[serde(default)]
    pub max_catch_up_ticks: Option<u32>,
    #[serde(default)]
    pub clamp_frame_delta_seconds: Option<f32>,
    #[serde(default)]
    pub presentation: Option<SceneFramePresentationDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneFramePresentationDocument {
    #[serde(default)]
    pub cache_game_frame: Option<bool>,
    #[serde(default)]
    pub hold_last_game_frame: Option<bool>,
    #[serde(default)]
    pub game_ui: Option<String>,
    #[serde(default)]
    pub devtools: Option<String>,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub debug_overlay: Option<String>,
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
