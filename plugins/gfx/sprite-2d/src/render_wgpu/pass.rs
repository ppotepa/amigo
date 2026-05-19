use amigo_plugin_api::{scene_alpha, scene_color, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite2dRenderTargetPlan {
    pub writes: Vec<TargetId>,
}

impl Sprite2dRenderTargetPlan {
    pub fn scene_color_alpha() -> Self {
        Self {
            writes: vec![scene_color(), scene_alpha()],
        }
    }
}
