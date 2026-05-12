use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{Text3dSceneService, queue_text3d_scene_command};

pub struct Text3dSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub text3d_scene_service: &'a Text3dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct Text3dSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub font: AssetKey,
}

pub fn can_handle_text3d_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueText3d { .. })
}

pub fn handle_text3d_scene_command(
    ctx: Text3dSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<Text3dSceneCommandOutcome> {
    match command {
        SceneCommand::QueueText3d { command } => {
            let entity =
                queue_text3d_scene_command(ctx.scene_service, ctx.text3d_scene_service, &command);
            ctx.scene_event_queue.publish(SceneEvent::Text3dQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                font: command.font.clone(),
            });
            Ok(Text3dSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                font: command.font,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "text-3d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
