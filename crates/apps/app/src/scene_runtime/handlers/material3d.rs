use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneMaterial3dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneMaterial3dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-material-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_3d_material::can_handle_material_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_3d_material::handle_material_scene_command(
            amigo_3d_material::MaterialSceneCommandContext {
                scene_service: ctx.scene_service,
                material_scene_service: ctx.material_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        if let Some(source) = outcome.source.as_ref() {
            crate::app_helpers::register_mod_asset_reference(
                ctx.asset_catalog,
                &outcome.source_mod,
                source,
                "3d",
                "material",
            );
        }

        ctx.dev_console_state.write_line(format!(
            "queued 3d material `{}` for entity `{}` from mod `{}`",
            outcome.material_label, outcome.entity_name, outcome.source_mod
        ));
        Ok(())
    }
}


