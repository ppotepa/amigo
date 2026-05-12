use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneTileMap2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneTileMap2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-tilemap-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_tilemap::can_handle_tilemap_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_tilemap::handle_tilemap_scene_command(
            amigo_2d_tilemap::TileMapSceneCommandContext {
                scene_service: ctx.scene_service,
                tilemap_scene_service: ctx.tilemap_scene_service,
                physics_scene_service: ctx.physics_scene_service,
                asset_catalog: ctx.asset_catalog,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;
        crate::app_helpers::register_mod_asset_reference(
            ctx.asset_catalog,
            &outcome.source_mod,
            &outcome.tileset,
            "2d",
            "tilemap",
        );
        if let Some(sheet_key) =
            crate::app_helpers::descriptor_first_tileset_spritesheet_key(&outcome.tileset)
        {
            crate::app_helpers::register_mod_asset_reference(
                ctx.asset_catalog,
                &outcome.source_mod,
                &sheet_key,
                "2d",
                "spritesheet",
            );
        }
        if let Some(ruleset) = outcome.ruleset.as_ref() {
            crate::app_helpers::register_mod_asset_reference(
                ctx.asset_catalog,
                &outcome.source_mod,
                ruleset,
                "2d",
                "tile-ruleset",
            );
        }
        ctx.dev_console_state.write_line(format!(
            "queued 2d tilemap entity `{}` from mod `{}` with tileset `{}`",
            outcome.entity_name,
            outcome.source_mod,
            outcome.tileset.as_str()
        ));

        Ok(())
    }
}


