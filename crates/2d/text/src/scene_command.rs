use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, Text2dSceneCommand,
    format_scene_command,
};

use crate::{Text2dSceneService, queue_text2d_scene_command};

pub struct Text2dSceneCommandHandler;

pub struct TextSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub text_scene_service: &'a Text2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct TextSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub font: AssetKey,
}

pub fn can_handle_text_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueText2d { .. })
}

pub fn handle_text_scene_command(
    ctx: TextSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<TextSceneCommandOutcome> {
    match command {
        SceneCommand::QueueText2d { command } => Ok(handle_queue_text_scene_command(ctx, command)),
        _ => Err(AmigoError::Message(format!(
            "text-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn handle_queue_text_scene_command(
    ctx: TextSceneCommandContext<'_>,
    command: Text2dSceneCommand,
) -> TextSceneCommandOutcome {
    let entity = queue_text2d_scene_command(ctx.scene_service, ctx.text_scene_service, &command);

    ctx.scene_event_queue.publish(SceneEvent::TextQueued {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
        font: command.font.clone(),
    });

    TextSceneCommandOutcome {
        entity_name: command.entity_name,
        source_mod: command.source_mod,
        font: command.font,
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Text2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_text_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let text_scene_service = runtime.required::<Text2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_text_scene_command(
            TextSceneCommandContext {
                scene_service: scene_service.as_ref(),
                text_scene_service: text_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;

        Ok(())
    }
}
