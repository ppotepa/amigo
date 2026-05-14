use amigo_assets::{AssetCatalog, AssetKey};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{SpriteSceneService, queue_sprite_scene_command, resolve_sprite_sheet_for_command};

pub struct Sprite2dSceneCommandHandler;

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
    matches!(command, SceneCommand::QueueSprite2d { .. })
}

pub fn handle_sprite_scene_command(
    ctx: SpriteSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<SpriteSceneCommandOutcome> {
    match command {
        SceneCommand::QueueSprite2d { command } => {
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
