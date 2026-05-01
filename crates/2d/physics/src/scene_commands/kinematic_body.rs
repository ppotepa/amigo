use amigo_scene::{KinematicBody2dSceneCommand, SceneEntityId, SceneService};

use crate::Physics2dSceneService;
use crate::model::{KinematicBody2d, KinematicBody2dCommand};

pub fn queue_kinematic_body_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &KinematicBody2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_body(KinematicBody2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        body: KinematicBody2d {
            velocity: command.velocity,
            gravity_scale: command.gravity_scale,
            terminal_velocity: command.terminal_velocity,
        },
    });
    entity
}
