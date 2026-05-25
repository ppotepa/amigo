use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    BOUNDS_2D_PLUGIN_SCENE_COMMAND_TYPE, Bounds2dSceneCommand, BoundsBehavior2dSceneCommand,
    ENTITY_POOL_PLUGIN_SCENE_COMMAND_TYPE, EntityPoolSceneCommand, EntityPoolSceneService,
    FREEFLIGHT_MOTION_2D_PLUGIN_SCENE_COMMAND_TYPE, FreeflightMotion2dSceneCommand,
    LIFETIME_PLUGIN_SCENE_COMMAND_TYPE, LifetimeSceneCommand, LifetimeSceneService,
    MOTION_CONTROLLER_2D_PLUGIN_SCENE_COMMAND_TYPE, MotionController2dSceneCommand,
    PROJECTILE_EMITTER_2D_PLUGIN_SCENE_COMMAND_TYPE, ProjectileEmitter2dSceneCommand,
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, VELOCITY_2D_PLUGIN_SCENE_COMMAND_TYPE,
    Velocity2dSceneCommand, format_scene_command,
};

use super::{
    Bounds2d, Bounds2dCommand, BoundsBehavior2d, FreeflightMotion2dCommand,
    FreeflightMotionProfile2d, FreeflightMotionState2d, Motion2dSceneService, MotionController2d,
    MotionController2dCommand, MotionProfile2d, ProjectileEmitter2d, ProjectileEmitter2dCommand,
    Velocity2d, Velocity2dCommand,
};

pub struct Motion2dSceneCommandHandler;

pub struct MotionSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub motion_scene_service: &'a Motion2dSceneService,
    pub entity_pool_scene_service: &'a EntityPoolSceneService,
    pub lifetime_scene_service: &'a LifetimeSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub enum MotionSceneCommandOutcome {
    MotionController {
        entity_name: String,
        source_mod: String,
    },
    EntityPool {
        pool: String,
        source_mod: String,
        member_count: usize,
    },
    Lifetime {
        entity_name: String,
        source_mod: String,
    },
    ProjectileEmitter {
        entity_name: String,
        source_mod: String,
    },
    Velocity {
        entity_name: String,
        source_mod: String,
    },
    Bounds {
        entity_name: String,
        source_mod: String,
    },
    Freeflight {
        entity_name: String,
        source_mod: String,
    },
}

pub fn can_handle_motion_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if matches!(
                command.command_type.as_str(),
                MOTION_CONTROLLER_2D_PLUGIN_SCENE_COMMAND_TYPE
                    | ENTITY_POOL_PLUGIN_SCENE_COMMAND_TYPE
                    | LIFETIME_PLUGIN_SCENE_COMMAND_TYPE
                    | PROJECTILE_EMITTER_2D_PLUGIN_SCENE_COMMAND_TYPE
                    | VELOCITY_2D_PLUGIN_SCENE_COMMAND_TYPE
                    | BOUNDS_2D_PLUGIN_SCENE_COMMAND_TYPE
                    | FREEFLIGHT_MOTION_2D_PLUGIN_SCENE_COMMAND_TYPE
            )
    )
}

pub fn handle_motion_scene_command(
    ctx: MotionSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<MotionSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == MOTION_CONTROLLER_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<MotionController2dSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "motion controller 2d plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.motion_scene_service
                .queue_motion_controller(MotionController2dCommand {
                    entity_id: entity,
                    entity_name: command.entity_name.clone(),
                    controller: MotionController2d {
                        params: MotionProfile2d {
                            max_speed: command.max_speed,
                            acceleration: command.acceleration,
                            deceleration: command.deceleration,
                            air_acceleration: command.air_acceleration,
                            gravity: command.gravity,
                            jump_velocity: command.jump_velocity,
                            terminal_velocity: command.terminal_velocity,
                        },
                    },
                });
            ctx.scene_event_queue
                .publish(SceneEvent::MotionControllerQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
            Ok(MotionSceneCommandOutcome::MotionController {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::Plugin { command } if command.command_type == ENTITY_POOL_PLUGIN_SCENE_COMMAND_TYPE => {
            let command = command
                .payload_as::<EntityPoolSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "entity pool plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            ctx.entity_pool_scene_service.queue(command.clone());
            ctx.scene_event_queue.publish(SceneEvent::EntityPoolQueued {
                pool: command.pool.clone(),
            });
            Ok(MotionSceneCommandOutcome::EntityPool {
                pool: command.pool,
                source_mod: command.source_mod,
                member_count: command.members.len(),
            })
        }
        SceneCommand::Plugin { command } if command.command_type == LIFETIME_PLUGIN_SCENE_COMMAND_TYPE => {
            let command = command
                .payload_as::<LifetimeSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "lifetime plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.lifetime_scene_service.queue(command.clone());
            ctx.scene_event_queue.publish(SceneEvent::LifetimeQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            Ok(MotionSceneCommandOutcome::Lifetime {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::Plugin { command }
            if command.command_type == PROJECTILE_EMITTER_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<ProjectileEmitter2dSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "projectile emitter 2d plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.motion_scene_service
                .queue_projectile_emitter(ProjectileEmitter2dCommand {
                    entity_id: entity,
                    entity_name: command.entity_name.clone(),
                    emitter: ProjectileEmitter2d {
                        pool: command.pool.clone(),
                        speed: command.speed,
                        spawn_offset: command.spawn_offset,
                        inherit_velocity_scale: command.inherit_velocity_scale,
                    },
                });
            ctx.scene_event_queue
                .publish(SceneEvent::ProjectileEmitterQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    pool: command.pool,
                });
            Ok(MotionSceneCommandOutcome::ProjectileEmitter {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::Plugin { command } if command.command_type == VELOCITY_2D_PLUGIN_SCENE_COMMAND_TYPE => {
            let command = command
                .payload_as::<Velocity2dSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "velocity 2d plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.motion_scene_service.queue_velocity(Velocity2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                velocity: Velocity2d::new(command.velocity),
            });
            ctx.scene_event_queue.publish(SceneEvent::Velocity2dQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            Ok(MotionSceneCommandOutcome::Velocity {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::Plugin { command } if command.command_type == BOUNDS_2D_PLUGIN_SCENE_COMMAND_TYPE => {
            let command = command
                .payload_as::<Bounds2dSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "bounds 2d plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.motion_scene_service.queue_bounds(Bounds2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                bounds: Bounds2d {
                    min: command.min,
                    max: command.max,
                    behavior: bounds_behavior_from_scene_command(command.behavior),
                },
            });
            ctx.scene_event_queue.publish(SceneEvent::Bounds2dQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            Ok(MotionSceneCommandOutcome::Bounds {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::Plugin { command }
            if command.command_type == FREEFLIGHT_MOTION_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<FreeflightMotion2dSceneCommand>()
                .ok_or_else(|| AmigoError::Message(
                    "freeflight motion 2d plugin scene command payload type mismatch".to_owned(),
                ))?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            let rotation_radians = ctx
                .scene_service
                .transform_of(&command.entity_name)
                .map(|transform| transform.rotation_euler.z)
                .unwrap_or(0.0);
            ctx.motion_scene_service
                .queue_freeflight(FreeflightMotion2dCommand {
                    entity_id: entity,
                    entity_name: command.entity_name.clone(),
                    profile: FreeflightMotionProfile2d {
                        thrust_acceleration: command.thrust_acceleration,
                        reverse_acceleration: command.reverse_acceleration,
                        strafe_acceleration: command.strafe_acceleration,
                        turn_acceleration: command.turn_acceleration,
                        linear_damping: command.linear_damping,
                        turn_damping: command.turn_damping,
                        max_speed: command.max_speed,
                        max_angular_speed: command.max_angular_speed,
                        thrust_response_curve: command.thrust_response_curve.clone(),
                        reverse_response_curve: command.reverse_response_curve.clone(),
                        strafe_response_curve: command.strafe_response_curve.clone(),
                        turn_response_curve: command.turn_response_curve.clone(),
                    },
                    initial_state: FreeflightMotionState2d {
                        velocity: command.initial_velocity,
                        angular_velocity: command.initial_angular_velocity,
                        rotation_radians,
                    },
                });
            ctx.scene_event_queue
                .publish(SceneEvent::FreeflightMotion2dQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
            Ok(MotionSceneCommandOutcome::Freeflight {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "motion-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn bounds_behavior_from_scene_command(behavior: BoundsBehavior2dSceneCommand) -> BoundsBehavior2d {
    match behavior {
        BoundsBehavior2dSceneCommand::Bounce { restitution } => {
            BoundsBehavior2d::Bounce { restitution }
        }
        BoundsBehavior2dSceneCommand::Wrap => BoundsBehavior2d::Wrap,
        BoundsBehavior2dSceneCommand::Hide => BoundsBehavior2d::Hide,
        BoundsBehavior2dSceneCommand::Despawn => BoundsBehavior2d::Despawn,
        BoundsBehavior2dSceneCommand::Clamp => BoundsBehavior2d::Clamp,
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Motion2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_motion_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let motion_scene_service = runtime.required::<Motion2dSceneService>()?;
        let entity_pool_scene_service = runtime.required::<EntityPoolSceneService>()?;
        let lifetime_scene_service = runtime.required::<LifetimeSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_motion_scene_command(
            MotionSceneCommandContext {
                scene_service: scene_service.as_ref(),
                motion_scene_service: motion_scene_service.as_ref(),
                entity_pool_scene_service: entity_pool_scene_service.as_ref(),
                lifetime_scene_service: lifetime_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
