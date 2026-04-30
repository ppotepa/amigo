use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_2d_motion::{
    Bounds2d, Bounds2dCommand, BoundsBehavior2d, FreeflightMotion2dCommand,
    FreeflightMotionProfile2d, FreeflightMotionState2d, MotionController2d,
    MotionController2dCommand, MotionProfile2d, ProjectileEmitter2d, ProjectileEmitter2dCommand,
    Velocity2d, Velocity2dCommand,
};
use amigo_scene::BoundsBehavior2dSceneCommand;

pub(crate) struct SceneMotion2dCommandHandler;

impl SceneCommandHandler for SceneMotion2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-motion-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueMotionController2d { .. }
                | SceneCommand::QueueEntityPool { .. }
                | SceneCommand::QueueLifetime { .. }
                | SceneCommand::QueueProjectileEmitter2d { .. }
                | SceneCommand::QueueVelocity2d { .. }
                | SceneCommand::QueueBounds2d { .. }
                | SceneCommand::QueueFreeflightMotion2d { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueMotionController2d { command } => {
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
                ctx.dev_console_state.write_line(format!(
                    "queued 2d motion controller `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueEntityPool { command } => {
                ctx.entity_pool_scene_service.queue(command.clone());
                ctx.scene_event_queue.publish(SceneEvent::EntityPoolQueued {
                    pool: command.pool.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued entity pool `{}` with {} members from mod `{}`",
                    command.pool,
                    command.members.len(),
                    command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueLifetime { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.lifetime_scene_service.queue(command.clone());
                ctx.scene_event_queue.publish(SceneEvent::LifetimeQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued lifetime `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueProjectileEmitter2d { command } => {
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
                        pool: command.pool.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d projectile emitter `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueVelocity2d { command } => {
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
                ctx.dev_console_state.write_line(format!(
                    "queued 2d velocity `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueBounds2d { command } => {
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
                ctx.dev_console_state.write_line(format!(
                    "queued 2d bounds `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueFreeflightMotion2d { command } => {
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
                ctx.dev_console_state.write_line(format!(
                    "queued 2d freeflight motion `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
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
