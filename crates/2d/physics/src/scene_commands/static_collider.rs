use amigo_scene::{SceneEntityId, SceneService, StaticCollider2dSceneCommand};

use crate::Physics2dSceneService;
use crate::model::{CollisionLayer, StaticCollider2d, StaticCollider2dCommand};

pub fn queue_static_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &StaticCollider2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_static_collider(StaticCollider2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: StaticCollider2d {
            size: command.size,
            offset: command.offset,
            layer: CollisionLayer::new(command.layer.clone()),
        },
    });
    entity
}
