use amigo_assets::{AssetCatalog, AssetKey};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{
    SpriteSceneService, queue_sprite_scene_command, resolve_sprite_sheet_for_command,
};

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
