use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_2d_particles::{ParticleEmitter2d, ParticleEmitter2dCommand};

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
                        emitter: ParticleEmitter2d::from_scene_command(&command),
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
