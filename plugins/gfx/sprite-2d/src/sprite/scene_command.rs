use std::any::Any;
use std::sync::Arc;

use amigo_assets::{AssetCatalog, AssetKey};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    format_scene_command, PluginSceneCommand, PluginSceneCommandPayload, SceneCommand, SceneEvent,
    SceneEventQueue, SceneService, Sprite2dSceneCommand,
};

use super::{queue_sprite_scene_command, resolve_sprite_sheet_for_command, SpriteSceneService};

pub struct Sprite2dSceneCommandHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dPluginCommandPayload(pub Sprite2dSceneCommand);

impl PluginSceneCommandPayload for Sprite2dPluginCommandPayload {
    fn command_type(&self) -> &'static str {
        "amigo.gfx.sprite-2d.scene-command.Sprite2D"
    }

    fn command_as_any(&self) -> &dyn Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Sprite2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn sprite_plugin_scene_command(command: Sprite2dSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(Sprite2dPluginCommandPayload(command)))
}

pub struct SpriteSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub sprite_scene_service: &'a SpriteSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
    pub asset_catalog: &'a AssetCatalog,
}

#[derive(Debug, Clone)]
pub struct SpriteSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub texture: AssetKey,
}

pub fn can_handle_sprite_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == "amigo.gfx.sprite-2d.scene-command.Sprite2D"
    )
}

pub fn handle_sprite_scene_command(
    ctx: SpriteSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<SpriteSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command } => {
            let Some(command) = command.payload_as::<Sprite2dSceneCommand>().cloned() else {
                return Err(AmigoError::Message(
                    "sprite-2d plugin command payload mismatch".to_owned(),
                ));
            };

            let resolved_sheet = resolve_sprite_sheet_for_command(ctx.asset_catalog, &command);
            let entity = queue_sprite_scene_command(
                ctx.scene_service,
                ctx.sprite_scene_service,
                &command,
                resolved_sheet,
            );

            ctx.scene_event_queue.publish(SceneEvent::SpriteQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                texture: command.texture.clone(),
            });

            Ok(SpriteSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                texture: command.texture,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "sprite-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Sprite2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_sprite_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let sprite_scene_service = runtime.required::<SpriteSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        let asset_catalog = runtime.required::<AssetCatalog>()?;

        handle_sprite_scene_command(
            SpriteSceneCommandContext {
                scene_service: scene_service.as_ref(),
                sprite_scene_service: sprite_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
                asset_catalog: asset_catalog.as_ref(),
            },
            command,
        )?;

        Ok(())
    }
}
