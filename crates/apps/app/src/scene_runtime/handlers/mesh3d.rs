use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneMesh3dCommandHandler;

impl SceneCommandHandler for SceneMesh3dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-mesh-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueMesh3d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueMesh3d { command } => {
                let entity = amigo_3d_mesh::queue_mesh_scene_command(
                    ctx.scene_service,
                    ctx.mesh_scene_service,
                    &command,
                );
                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.mesh_asset,
                    "3d",
                    "mesh",
                );
                ctx.scene_event_queue.publish(SceneEvent::MeshQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    mesh_asset: command.mesh_asset.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 3d mesh entity `{}` from mod `{}` with mesh `{}`",
                    command.entity_name,
                    command.source_mod,
                    command.mesh_asset.as_str()
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
