use amigo_scene::{CircleCollider2dSceneCommand, SceneEntityId, SceneService};

use crate::Physics2dSceneService;
use crate::model::{CircleCollider2d, CircleCollider2dCommand};

pub fn queue_circle_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &CircleCollider2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_circle_collider(CircleCollider2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: CircleCollider2d {
            radius: command.radius.max(0.0),
            offset: command.offset,
        },
    });
    entity
}
