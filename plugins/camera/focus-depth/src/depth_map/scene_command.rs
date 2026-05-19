use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{
    DepthMap2dSceneService, queue_depth_aux_map2d_scene_command, queue_depth_map2d_scene_command,
};

pub struct DepthMap2dSceneCommandHandler;

pub struct DepthMap2dSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub depth_map_scene_service: &'a DepthMap2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct DepthMap2dSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub asset: AssetKey,
}

pub fn can_handle_depth_map2d_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueDepthMap2d { .. } | SceneCommand::QueueDepthAuxMap2d { .. }
    )
}

pub fn handle_depth_map2d_scene_command(
    ctx: DepthMap2dSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<DepthMap2dSceneCommandOutcome> {
    match command {
        SceneCommand::QueueDepthMap2d { command } => {
            let entity = queue_depth_map2d_scene_command(
                ctx.scene_service,
                ctx.depth_map_scene_service,
                &command,
            );

            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });

            Ok(DepthMap2dSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                asset: command.asset,
            })
        }
        SceneCommand::QueueDepthAuxMap2d { command } => {
            let entity = queue_depth_aux_map2d_scene_command(
                ctx.scene_service,
                ctx.depth_map_scene_service,
                &command,
            );

            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });

            Ok(DepthMap2dSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                asset: command.asset,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "depth-map-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for DepthMap2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_depth_map2d_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let depth_map_scene_service = runtime.required::<DepthMap2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_depth_map2d_scene_command(
            DepthMap2dSceneCommandContext {
                scene_service: scene_service.as_ref(),
                depth_map_scene_service: depth_map_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;

        Ok(())
    }
}
