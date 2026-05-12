use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneText3dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneText3dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-text-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_3d_text::can_handle_text3d_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_3d_text::handle_text3d_scene_command(
            amigo_3d_text::Text3dSceneCommandContext {
                scene_service: ctx.scene_service,
                text3d_scene_service: ctx.text3d_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        crate::app_helpers::register_mod_asset_reference(
            ctx.asset_catalog,
            &outcome.source_mod,
            &outcome.font,
            "3d",
            "text",
        );
        ctx.dev_console_state.write_line(format!(
            "queued 3d text entity `{}` from mod `{}` with font `{}`",
            outcome.entity_name,
            outcome.source_mod,
            outcome.font.as_str()
        ));
        Ok(())
    }
}


