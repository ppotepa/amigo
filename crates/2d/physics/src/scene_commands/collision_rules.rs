use amigo_scene::CollisionEventRule2dSceneCommand;

use crate::Physics2dSceneService;
use crate::model::{CollisionEventRule2d, CollisionEventRule2dCommand};

pub fn queue_collision_event_rule_scene_command(
    physics_scene_service: &Physics2dSceneService,
    command: &CollisionEventRule2dSceneCommand,
) {
    physics_scene_service.queue_collision_event_rule(CollisionEventRule2dCommand {
        source_mod: command.source_mod.clone(),
        rule: CollisionEventRule2d {
            id: command.id.clone(),
            source: command.source.clone(),
            target: command.target.clone(),
            event: command.event.clone(),
            once_per_overlap: command.once_per_overlap,
        },
    });
}
