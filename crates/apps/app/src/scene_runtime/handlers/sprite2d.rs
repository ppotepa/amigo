use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneSprite2dCommandHandler;

impl SceneCommandHandler for SceneSprite2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-sprite-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueSprite2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueSprite2d { command } => {
                let resolved_sheet =
                    assets::resolve_sprite_sheet_for_command(ctx.asset_catalog, &command);
                let entity = amigo_2d_sprite::queue_sprite_scene_command(
                    ctx.scene_service,
                    ctx.sprite_scene_service,
                    &command,
                    resolved_sheet,
                );
                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.texture,
                    "2d",
                    "sprite",
                );
                ctx.scene_event_queue.publish(SceneEvent::SpriteQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    texture: command.texture.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d sprite entity `{}` from mod `{}` with asset `{}`",
                    command.entity_name,
                    command.source_mod,
                    command.texture.as_str()
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
