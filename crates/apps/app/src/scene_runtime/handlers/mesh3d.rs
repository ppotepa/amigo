use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneMesh3dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneMesh3dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-mesh-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_3d_mesh::can_handle_mesh_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_3d_mesh::handle_mesh_scene_command(
            amigo_3d_mesh::MeshSceneCommandContext {
                scene_service: ctx.scene_service,
                mesh_scene_service: ctx.mesh_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        crate::app_helpers::register_mod_asset_reference(
            ctx.asset_catalog,
            &outcome.source_mod,
            &outcome.mesh_asset,
            "3d",
            "mesh",
        );
        ctx.dev_console_state.write_line(format!(
            "queued 3d mesh entity `{}` from mod `{}` with mesh `{}`",
            outcome.entity_name,
            outcome.source_mod,
            outcome.mesh_asset.as_str()
        ));
        Ok(())
    }
}


