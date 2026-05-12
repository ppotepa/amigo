use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{MaterialSceneService, queue_material_scene_command};

pub struct MaterialSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub material_scene_service: &'a MaterialSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct MaterialSceneCommandOutcome {
    pub entity_name: String,
    pub material_label: String,
    pub source_mod: String,
    pub source: Option<AssetKey>,
}

pub fn can_handle_material_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueMaterial3d { .. })
}

pub fn handle_material_scene_command(
    ctx: MaterialSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<MaterialSceneCommandOutcome> {
    match command {
        SceneCommand::QueueMaterial3d { command } => {
            let entity =
                queue_material_scene_command(ctx.scene_service, ctx.material_scene_service, &command);
            ctx.scene_event_queue.publish(SceneEvent::MaterialQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                material_label: command.label.clone(),
            });
            Ok(MaterialSceneCommandOutcome {
                entity_name: command.entity_name,
                material_label: command.label,
                source_mod: command.source_mod,
                source: command.source,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "material-3d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
