use amigo_scene::{DepthMap2dSceneCommand, SceneEntityId, SceneService};

use crate::{DepthMap2dDrawCommand, DepthMap2dInstance, DepthMap2dSceneService};

pub fn queue_depth_map2d_scene_command(
    scene_service: &SceneService,
    depth_map_scene_service: &DepthMap2dSceneService,
    command: &DepthMap2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());

    depth_map_scene_service.queue(DepthMap2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        depth_map: DepthMap2dInstance {
            id: command.id.clone(),
            asset: command.asset.clone(),
            size: command.size,
            viewport_fit: command.viewport_fit.into(),
            white_is_near: command.white_is_near,
        },
        z_index: command.z_index,
        transform: command.transform,
    });

    entity
}
