use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{Text3dSceneService, queue_text3d_scene_command};

pub struct Text3dSceneCommandHandler;

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
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TEXT_3D_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_text3d_scene_command(
    ctx: Text3dSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<Text3dSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TEXT_3D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(command) = command
                .payload_as::<amigo_scene::Text3dSceneCommand>()
                .cloned()
            else {
                return Err(AmigoError::Message(
                    "text-3d plugin command payload mismatch".to_owned(),
                ));
            };
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

impl amigo_scene::RuntimeSceneCommandHandler for Text3dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_text3d_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let text3d_scene_service = runtime.required::<Text3dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_text3d_scene_command(
            Text3dSceneCommandContext {
                scene_service: scene_service.as_ref(),
                text3d_scene_service: text3d_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
