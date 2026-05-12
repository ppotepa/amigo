use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneSprite2dCommandHandler;

impl SceneCommandHandler for SceneSprite2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-sprite-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_sprite::can_handle_sprite_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_sprite::handle_sprite_scene_command(
            amigo_2d_sprite::SpriteSceneCommandContext {
                scene_service: ctx.scene_service,
                sprite_scene_service: ctx.sprite_scene_service,
                scene_event_queue: ctx.scene_event_queue,
                asset_catalog: ctx.asset_catalog,
            },
            command,
        )?;
        crate::app_helpers::register_mod_asset_reference(
            ctx.asset_catalog,
            &outcome.source_mod,
            &outcome.texture,
            "2d",
            "sprite",
        );
        ctx.dev_console_state.write_line(format!(
            "queued 2d sprite entity `{}` from mod `{}` with asset `{}`",
            outcome.entity_name,
            outcome.source_mod,
            outcome.texture.as_str()
        ));

        Ok(())
    }
}
