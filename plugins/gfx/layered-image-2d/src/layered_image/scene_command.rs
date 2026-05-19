use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use super::{LayeredImageSceneService, queue_layered_image_scene_command};

pub struct LayeredImage2dSceneCommandHandler;

pub struct LayeredImageSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub layered_image_scene_service: &'a LayeredImageSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct LayeredImageSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub asset: AssetKey,
}

pub fn can_handle_layered_image_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueLayeredImage2d { .. })
}

pub fn handle_layered_image_scene_command(
    ctx: LayeredImageSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<LayeredImageSceneCommandOutcome> {
    match command {
        SceneCommand::QueueLayeredImage2d { command } => {
            let entity = queue_layered_image_scene_command(
                ctx.scene_service,
                ctx.layered_image_scene_service,
                &command,
            );

            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });

            Ok(LayeredImageSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                asset: command.asset,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "layered-image-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for LayeredImage2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_layered_image_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let layered_image_scene_service = runtime.required::<LayeredImageSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_layered_image_scene_command(
            LayeredImageSceneCommandContext {
                scene_service: scene_service.as_ref(),
                layered_image_scene_service: layered_image_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;

        Ok(())
    }
}
