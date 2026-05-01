use amigo_scene::{AabbCollider2dSceneCommand, SceneEntityId, SceneService};

use crate::Physics2dSceneService;
use crate::model::{AabbCollider2d, AabbCollider2dCommand, CollisionLayer, CollisionMask};

pub fn queue_aabb_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &AabbCollider2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_aabb_collider(AabbCollider2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: AabbCollider2d {
            size: command.size,
            offset: command.offset,
            layer: CollisionLayer::new(command.layer.clone()),
            mask: CollisionMask::new(
                command
                    .mask
                    .iter()
                    .cloned()
                    .map(CollisionLayer::new)
                    .collect::<Vec<_>>(),
            ),
        },
    });
    entity
}
