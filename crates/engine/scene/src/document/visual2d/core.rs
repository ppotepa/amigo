use super::{LightGroup2dDocument, LightRoute2dDocument, PostFx2dDocument, RenderLayer2dDocument};
use serde::{Deserialize, Serialize};

pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_one() -> f32 {
    1.0
}

pub(super) fn default_white_color() -> String {
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
    #[serde(default)]
    pub post_fx: Vec<PostFx2dDocument>,
}
