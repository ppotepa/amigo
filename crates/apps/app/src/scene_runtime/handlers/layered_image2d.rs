use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneLayeredImage2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneLayeredImage2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-layered-image-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_layered_image::can_handle_layered_image_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_layered_image::handle_layered_image_scene_command(
            amigo_2d_layered_image::LayeredImageSceneCommandContext {
                scene_service: ctx.scene_service,
                layered_image_scene_service: ctx.layered_image_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        crate::app_helpers::register_mod_asset_reference(
            ctx.asset_catalog,
            &outcome.source_mod,
            &outcome.asset,
            "2d",
            "layered-image",
        );

        ctx.dev_console_state.write_line(format!(
            "queued layered image entity `{}` from mod `{}` with asset `{}`",
            outcome.entity_name,
            outcome.source_mod,
            outcome.asset.as_str()
        ));

        Ok(())
    }
}


