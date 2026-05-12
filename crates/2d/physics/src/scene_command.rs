use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command,
};

use crate::{
    Physics2dSceneService, queue_aabb_collider_scene_command,
    queue_circle_collider_scene_command, queue_collision_event_rule_scene_command,
    queue_kinematic_body_scene_command, queue_static_collider_scene_command,
    queue_trigger_scene_command,
};

pub struct Physics2dSceneCommandHandler;

pub struct PhysicsSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub physics_scene_service: &'a Physics2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub enum PhysicsSceneCommandOutcome {
    KinematicBody { entity_name: String, source_mod: String },
    AabbCollider { entity_name: String, source_mod: String },
    StaticCollider { entity_name: String, source_mod: String },
    CircleCollider { entity_name: String, source_mod: String },
    Trigger {
        entity_name: String,
        source_mod: String,
        topic: Option<String>,
    },
    CollisionEventRule {
        id: String,
        source_mod: String,
        topic: String,
    },
}

pub fn can_handle_physics_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueKinematicBody2d { .. }
            | SceneCommand::QueueAabbCollider2d { .. }
            | SceneCommand::QueueStaticCollider2d { .. }
            | SceneCommand::QueueCircleCollider2d { .. }
            | SceneCommand::QueueTrigger2d { .. }
            | SceneCommand::QueueCollisionEventRule2d { .. }
    )
}

pub fn handle_physics_scene_command(
    ctx: PhysicsSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<PhysicsSceneCommandOutcome> {
    match command {
        SceneCommand::QueueKinematicBody2d { command } => {
            let entity = queue_kinematic_body_scene_command(
                ctx.scene_service,
                ctx.physics_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::KinematicBodyQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            Ok(PhysicsSceneCommandOutcome::KinematicBody {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueAabbCollider2d { command } => {
            let entity = queue_aabb_collider_scene_command(
                ctx.scene_service,
                ctx.physics_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::AabbColliderQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            Ok(PhysicsSceneCommandOutcome::AabbCollider {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueStaticCollider2d { command } => {
            let entity = queue_static_collider_scene_command(
                ctx.scene_service,
                ctx.physics_scene_service,
                &command,
            );
            ctx.scene_event_queue
                .publish(SceneEvent::StaticColliderQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
            Ok(PhysicsSceneCommandOutcome::StaticCollider {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueCircleCollider2d { command } => {
            let entity = queue_circle_collider_scene_command(
                ctx.scene_service,
                ctx.physics_scene_service,
                &command,
            );
            ctx.scene_event_queue
                .publish(SceneEvent::CircleColliderQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
            Ok(PhysicsSceneCommandOutcome::CircleCollider {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueTrigger2d { command } => {
            let entity =
                queue_trigger_scene_command(ctx.scene_service, ctx.physics_scene_service, &command);
            ctx.scene_event_queue.publish(SceneEvent::TriggerQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                topic: command.event.clone(),
            });
            Ok(PhysicsSceneCommandOutcome::Trigger {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                topic: command.event,
            })
        }
        SceneCommand::QueueCollisionEventRule2d { command } => {
            queue_collision_event_rule_scene_command(ctx.physics_scene_service, &command);
            ctx.scene_event_queue
                .publish(SceneEvent::CollisionEventRuleQueued {
                    rule_id: command.id.clone(),
                    topic: command.event.clone(),
                });
            Ok(PhysicsSceneCommandOutcome::CollisionEventRule {
                id: command.id,
                source_mod: command.source_mod,
                topic: command.event,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "physics-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Physics2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_physics_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let physics_scene_service = runtime.required::<Physics2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_physics_scene_command(
            PhysicsSceneCommandContext {
                scene_service: scene_service.as_ref(),
                physics_scene_service: physics_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
