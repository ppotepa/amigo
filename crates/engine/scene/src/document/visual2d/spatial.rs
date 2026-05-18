use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Visual2dSpatialDocument {
    #[serde(default)]
    pub depth_space: DepthSpace2dDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DepthCurve2dDocument {
    Linear,
    #[default]
    Logarithmic,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthSpace2dDocument {
    #[serde(default = "default_depth_near_m")]
    pub near_m: f32,
    #[serde(default = "default_depth_far_m")]
    pub far_m: f32,
    #[serde(default)]
    pub curve: DepthCurve2dDocument,
}

impl Default for DepthSpace2dDocument {
    fn default() -> Self {
        Self {
            near_m: default_depth_near_m(),
            far_m: default_depth_far_m(),
            curve: DepthCurve2dDocument::Logarithmic,
        }
    }
}

impl DepthSpace2dDocument {
    pub fn to_runtime(self) -> amigo_2d_spatial::DepthSpace2d {
        amigo_2d_spatial::DepthSpace2d {
            near_m: self.near_m,
            far_m: self.far_m,
            curve: match self.curve {
                DepthCurve2dDocument::Linear => amigo_2d_spatial::DepthCurve2d::Linear,
                DepthCurve2dDocument::Logarithmic => amigo_2d_spatial::DepthCurve2d::Logarithmic,
            },
        }
        .normalized()
    }
}

fn default_depth_near_m() -> f32 {
    1.0
}

fn default_depth_far_m() -> f32 {
    1500.0
}
