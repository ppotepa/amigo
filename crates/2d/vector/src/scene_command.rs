use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, VectorShape2dSceneCommand,
    format_scene_command,
};

use crate::{VectorSceneService, queue_vector_shape_scene_command};

pub struct Vector2dSceneCommandHandler;

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
}

pub fn handle_vector_scene_command(
    ctx: VectorSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<VectorSceneCommandOutcome> {
    match command {
        SceneCommand::QueueVectorShape2d { command } => {
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
    let entity = queue_vector_shape_scene_command(
        ctx.scene_service,
        ctx.vector_scene_service,
        &command,
    );

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

