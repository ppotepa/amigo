use amigo_scene::{SceneEntityId, SceneService, Trigger2dSceneCommand};

use crate::Physics2dSceneService;
use crate::model::{CollisionLayer, CollisionMask, Trigger2d, Trigger2dCommand};

pub fn queue_trigger_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &Trigger2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_trigger(Trigger2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        trigger: Trigger2d {
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
            topic: command.event.clone(),
        },
    });
    entity
}
