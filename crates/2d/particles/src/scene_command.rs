use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command,
};

use crate::{Particle2dSceneService, ParticleEmitter2d, ParticleEmitter2dCommand};

pub struct ParticlesSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub particle2d_scene_service: &'a Particle2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct ParticlesSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
}

pub fn can_handle_particles_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueParticleEmitter2d { .. })
}

pub fn handle_particles_scene_command(
    ctx: ParticlesSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<ParticlesSceneCommandOutcome> {
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
            Ok(ParticlesSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "particles-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
