use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthMapViewportFit2d {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

impl From<amigo_scene::DepthMapViewportFit2dSceneCommand> for DepthMapViewportFit2d {
    fn from(value: amigo_scene::DepthMapViewportFit2dSceneCommand) -> Self {
        match value {
            amigo_scene::DepthMapViewportFit2dSceneCommand::Fixed => Self::Fixed,
            amigo_scene::DepthMapViewportFit2dSceneCommand::Stretch => Self::Stretch,
            amigo_scene::DepthMapViewportFit2dSceneCommand::Contain => Self::Contain,
            amigo_scene::DepthMapViewportFit2dSceneCommand::Cover => Self::Cover,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthMap2dInstance {
    pub id: String,
    pub asset: AssetKey,
    pub size: Vec2,
    pub viewport_fit: DepthMapViewportFit2d,
    pub white_is_near: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthMap2dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub depth_map: DepthMap2dInstance,
    pub z_index: f32,
    pub transform: Transform2,
}
