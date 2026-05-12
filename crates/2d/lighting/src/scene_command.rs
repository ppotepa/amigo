use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command,
};

use crate::{
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
    queue_global_light_2d_scene_command, queue_light_group_2d_scene_command,
    queue_lightmap_2d_source_scene_command,
};

pub struct LightingSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub enum LightingSceneCommandOutcome {
    GlobalLight {
        id: String,
        entity_name: String,
        source_mod: String,
    },
    LightMapSource {
        id: String,
        entity_name: String,
        source_mod: String,
        channel_count: usize,
    },
    LightGroup {
        id: String,
        source_mod: String,
    },
}

pub fn can_handle_lighting_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueGlobalLight2d { .. }
            | SceneCommand::QueueLightMap2dSource { .. }
            | SceneCommand::QueueLightGroup2d { .. }
    )
}

pub fn handle_lighting_scene_command(
    ctx: LightingSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<LightingSceneCommandOutcome> {
    match command {
        SceneCommand::QueueGlobalLight2d { command } => {
            let entity = queue_global_light_2d_scene_command(
                ctx.scene_service,
                ctx.global_light2d_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });
            Ok(LightingSceneCommandOutcome::GlobalLight {
                id: command.id,
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueLightMap2dSource { command } => {
            let entity = queue_lightmap_2d_source_scene_command(
                ctx.scene_service,
                ctx.lightmap2d_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });
            Ok(LightingSceneCommandOutcome::LightMapSource {
                id: command.id,
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                channel_count: command.channels.len(),
            })
        }
        SceneCommand::QueueLightGroup2d { command } => {
            let id = command.id.clone();
            let source_mod = command.source_mod.clone();
            queue_light_group_2d_scene_command(ctx.light_group2d_scene_service, command);
            Ok(LightingSceneCommandOutcome::LightGroup { id, source_mod })
        }
        _ => Err(AmigoError::Message(format!(
            "lighting-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
