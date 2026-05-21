use std::any::Any;
use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    PluginSceneCommand, PluginSceneCommandPayload, SceneCommand, SceneEvent, SceneEventQueue,
    SceneService, VectorShape2dSceneCommand, format_scene_command,
};

use super::{VectorSceneService, queue_vector_shape_scene_command};

pub struct Vector2dSceneCommandHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dPluginCommandPayload(pub VectorShape2dSceneCommand);

impl PluginSceneCommandPayload for VectorShape2dPluginCommandPayload {
    fn command_type(&self) -> &'static str {
        "amigo.gfx.vector-2d.scene-command.VectorShape2D"
    }

    fn command_as_any(&self) -> &dyn Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<VectorShape2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn vector_plugin_scene_command(command: VectorShape2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(VectorShape2dPluginCommandPayload(command)))
}

pub struct VectorSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub vector_scene_service: &'a VectorSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct VectorSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
}

pub fn can_handle_vector_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueVectorShape2d { .. })
        || matches!(
            command,
            SceneCommand::Plugin { command }
                if command.command_type == "amigo.gfx.vector-2d.scene-command.VectorShape2D"
        )
}

pub fn handle_vector_scene_command(
    ctx: VectorSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<VectorSceneCommandOutcome> {
    match command {
        SceneCommand::QueueVectorShape2d { command } => {
            Ok(handle_queue_vector_shape_scene_command(ctx, command))
        }
        SceneCommand::Plugin { command } => {
            let Some(command) = command.vector_shape_2d_command().cloned() else {
                return Err(AmigoError::Message(format!(
                "vector-2d cannot handle command {}",
                format_scene_command(&SceneCommand::Plugin { command })
                )));
            };
            Ok(handle_queue_vector_shape_scene_command(ctx, command))
        }
        _ => Err(AmigoError::Message(format!(
            "vector-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn handle_queue_vector_shape_scene_command(
    ctx: VectorSceneCommandContext<'_>,
    command: VectorShape2dSceneCommand,
) -> VectorSceneCommandOutcome {
    let entity =
        queue_vector_shape_scene_command(ctx.scene_service, ctx.vector_scene_service, &command);

    ctx.scene_event_queue.publish(SceneEvent::VectorQueued {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
    });

    VectorSceneCommandOutcome {
        entity_name: command.entity_name,
        source_mod: command.source_mod,
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Vector2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_vector_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let vector_scene_service = runtime.required::<VectorSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_vector_scene_command(
            VectorSceneCommandContext {
                scene_service: scene_service.as_ref(),
                vector_scene_service: vector_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;

        Ok(())
    }
}
