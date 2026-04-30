use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_2d_particles::{
    ParticleEmitter2d, ParticleEmitter2dCommand, ParticleForce2d, ParticleShape2d,
    ParticleSpawnArea2d,
};
use amigo_scene::{
    ParticleForce2dSceneCommand, ParticleShape2dSceneCommand, ParticleSpawnArea2dSceneCommand,
};

pub(crate) struct SceneParticles2dCommandHandler;

impl SceneCommandHandler for SceneParticles2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-particles-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueParticleEmitter2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueParticleEmitter2d { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.particle2d_scene_service
                    .queue_emitter(ParticleEmitter2dCommand {
                        entity_id: entity,
                        entity_name: command.entity_name.clone(),
                        emitter: ParticleEmitter2d {
                            attached_to: command.attached_to.clone(),
                            local_offset: command.local_offset,
                            local_direction_radians: command.local_direction_radians,
                            spawn_area: particle_spawn_area_from_scene_command(command.spawn_area),
                            active: command.active,
                            spawn_rate: command.spawn_rate,
                            max_particles: command.max_particles,
                            particle_lifetime: command.particle_lifetime,
                            lifetime_jitter: command.lifetime_jitter,
                            initial_speed: command.initial_speed,
                            speed_jitter: command.speed_jitter,
                            spread_radians: command.spread_radians,
                            inherit_parent_velocity: command.inherit_parent_velocity,
                            initial_size: command.initial_size,
                            final_size: command.final_size,
                            color: command.color,
                            color_ramp: command.color_ramp.clone(),
                            z_index: command.z_index,
                            shape: particle_shape_from_scene_command(command.shape),
                            emission_rate_curve: command.emission_rate_curve.clone(),
                            size_curve: command.size_curve.clone(),
                            alpha_curve: command.alpha_curve.clone(),
                            speed_curve: command.speed_curve.clone(),
                            forces: command
                                .forces
                                .iter()
                                .copied()
                                .map(particle_force_from_scene_command)
                                .collect(),
                        },
                    });
                ctx.scene_event_queue
                    .publish(SceneEvent::ParticleEmitterQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d particle emitter `{}` from mod `{}`",
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

fn particle_shape_from_scene_command(shape: ParticleShape2dSceneCommand) -> ParticleShape2d {
    match shape {
        ParticleShape2dSceneCommand::Circle { segments } => ParticleShape2d::Circle { segments },
        ParticleShape2dSceneCommand::Quad => ParticleShape2d::Quad,
        ParticleShape2dSceneCommand::Line { length } => ParticleShape2d::Line { length },
    }
}

fn particle_spawn_area_from_scene_command(
    spawn_area: ParticleSpawnArea2dSceneCommand,
) -> ParticleSpawnArea2d {
    match spawn_area {
        ParticleSpawnArea2dSceneCommand::Point => ParticleSpawnArea2d::Point,
        ParticleSpawnArea2dSceneCommand::Line { length } => ParticleSpawnArea2d::Line { length },
        ParticleSpawnArea2dSceneCommand::Rect { size } => ParticleSpawnArea2d::Rect { size },
        ParticleSpawnArea2dSceneCommand::Circle { radius } => {
            ParticleSpawnArea2d::Circle { radius }
        }
        ParticleSpawnArea2dSceneCommand::Ring {
            inner_radius,
            outer_radius,
        } => ParticleSpawnArea2d::Ring {
            inner_radius,
            outer_radius,
        },
    }
}

fn particle_force_from_scene_command(force: ParticleForce2dSceneCommand) -> ParticleForce2d {
    match force {
        ParticleForce2dSceneCommand::Gravity { acceleration } => {
            ParticleForce2d::Gravity { acceleration }
        }
        ParticleForce2dSceneCommand::ConstantAcceleration { acceleration } => {
            ParticleForce2d::ConstantAcceleration { acceleration }
        }
        ParticleForce2dSceneCommand::Drag { coefficient } => ParticleForce2d::Drag { coefficient },
        ParticleForce2dSceneCommand::Wind { velocity, strength } => {
            ParticleForce2d::Wind { velocity, strength }
        }
    }
}
