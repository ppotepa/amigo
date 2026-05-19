use serde::{Deserialize, Serialize};

use amigo_camera_optics_plugin::scene::CameraOpticalResponse2dDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material2dOpticalModeDocument {
    Opaque,
    Transmissive,
    Refractive,
    Emissive,
}

impl Default for Material2dOpticalModeDocument {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Material2dOpticalDocument {
    #[serde(default)]
    pub mode: Material2dOpticalModeDocument,
    #[serde(default)]
    pub transmission: f32,
    #[serde(default)]
    pub refraction_px: f32,
    #[serde(default)]
    pub distortion: f32,
    #[serde(default)]
    pub dispersion: f32,
    #[serde(default)]
    pub roughness: f32,
    #[serde(default)]
    pub edge_boost: f32,
}

impl Default for Material2dOpticalDocument {
    fn default() -> Self {
        Self {
            mode: Material2dOpticalModeDocument::Opaque,
            transmission: 0.0,
            refraction_px: 0.0,
            distortion: 0.0,
            dispersion: 0.0,
            roughness: 0.0,
            edge_boost: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Material2dLightingDocument {
    #[serde(default)]
    pub receives_light: bool,
    #[serde(default)]
    pub response: f32,
}

impl Default for Material2dLightingDocument {
    fn default() -> Self {
        Self {
            receives_light: false,
            response: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Material2dDocument {
    #[serde(default)]
    pub optical: Material2dOpticalDocument,
    #[serde(default)]
    pub lighting: Material2dLightingDocument,
    #[serde(default)]
    pub camera_response: CameraOpticalResponse2dDocument,
}
