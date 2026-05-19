use serde::{Deserialize, Serialize};

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
