use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{MeshSceneService, queue_mesh_scene_command};

pub struct MeshSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub mesh_scene_service: &'a MeshSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct MeshSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub mesh_asset: AssetKey,
}

pub fn can_handle_mesh_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueMesh3d { .. })
}

pub fn handle_mesh_scene_command(
    ctx: MeshSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<MeshSceneCommandOutcome> {
    match command {
        SceneCommand::QueueMesh3d { command } => {
            let entity = queue_mesh_scene_command(ctx.scene_service, ctx.mesh_scene_service, &command);
            ctx.scene_event_queue.publish(SceneEvent::MeshQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                mesh_asset: command.mesh_asset.clone(),
            });
            Ok(MeshSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                mesh_asset: command.mesh_asset,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "mesh-3d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
