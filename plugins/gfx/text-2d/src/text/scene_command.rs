use std::any::Any;
use std::sync::Arc;

use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    PluginSceneCommand, PluginSceneCommandPayload, SceneCommand, SceneEvent, SceneEventQueue,
    SceneService, Text2dSceneCommand, format_scene_command,
};

use crate::{queue_text2d_scene_command, Text2dSceneService};

pub struct Text2dSceneCommandHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct Text2dPluginCommandPayload(pub Text2dSceneCommand);

impl PluginSceneCommandPayload for Text2dPluginCommandPayload {
    fn command_type(&self) -> &'static str {
        "amigo.gfx.text-2d.scene-command.Text2D"
    }

    fn command_as_any(&self) -> &dyn Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Text2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn text_plugin_scene_command(command: Text2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Text2dPluginCommandPayload(command)))
}

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
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == "amigo.gfx.text-2d.scene-command.Text2D"
    )
}

pub fn handle_text_scene_command(
    ctx: TextSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<TextSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command } => {
            let Some(command) = command.payload_as::<Text2dSceneCommand>().cloned() else {
                return Err(AmigoError::Message(format!(
                "text-2d cannot handle command {}",
                format_scene_command(&SceneCommand::Plugin { command })
                )));
            };
            Ok(handle_queue_text_scene_command(ctx, command))
        }
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
