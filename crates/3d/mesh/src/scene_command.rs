use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{MeshSceneService, queue_mesh_scene_command};

pub struct Mesh3dSceneCommandHandler;

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
            let entity =
                queue_mesh_scene_command(ctx.scene_service, ctx.mesh_scene_service, &command);
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

impl amigo_scene::RuntimeSceneCommandHandler for Mesh3dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_mesh_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let mesh_scene_service = runtime.required::<MeshSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_mesh_scene_command(
            MeshSceneCommandContext {
                scene_service: scene_service.as_ref(),
                mesh_scene_service: mesh_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
