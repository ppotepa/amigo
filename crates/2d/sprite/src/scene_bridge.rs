use crate::model::{Sprite, SpriteAnimationOverride, SpriteDrawCommand, SpriteSheet};
use crate::service::SpriteSceneService;
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_math::Vec2;
use amigo_scene::{
    SceneEntityId, SceneService, Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
};

pub fn queue_sprite_scene_command(
    scene_service: &SceneService,
    sprite_scene_service: &SpriteSceneService,
    command: &Sprite2dSceneCommand,
    resolved_sheet: Option<SpriteSheet>,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    sprite_scene_service.queue(SpriteDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        sprite: Sprite {
            texture: command.texture.clone(),
            size: command.size,
            sheet: resolved_sheet,
            sheet_is_explicit: command.sheet.is_some(),
            animation_override: command.animation.as_ref().map(|animation| {
                SpriteAnimationOverride {
                    fps: animation.fps,
                    looping: animation.looping,
                    start_frame: animation.start_frame,
                }
            }),
            frame_index: command
                .animation
                .as_ref()
                .and_then(|animation| animation.start_frame)
                .unwrap_or(0),
            frame_elapsed: 0.0,
        },
        z_index: command.z_index,
        transform: command.transform,
    });
    entity
}

pub fn infer_sprite_sheet_from_prepared_asset(prepared: &PreparedAsset) -> Option<SpriteSheet> {
    if !matches!(prepared.kind, PreparedAssetKind::SpriteSheet2d) {
        return None;
    }

    let columns = prepared
        .metadata
        .get("columns")?
        .parse::<u32>()
        .ok()?
        .max(1);
    let rows = prepared.metadata.get("rows")?.parse::<u32>().ok()?.max(1);
    let frame_width = prepared.metadata.get("frame_size.x")?.parse::<f32>().ok()?;
    let frame_height = prepared.metadata.get("frame_size.y")?.parse::<f32>().ok()?;
    let fps = prepared
        .metadata
        .get("fps")
        .and_then(|value| value.parse::<f32>().ok())
        .or_else(|| first_animation_f32(prepared, "fps"))
        .unwrap_or(0.0);
    let looping = prepared
        .metadata
        .get("looping")
        .and_then(|value| value.parse::<bool>().ok())
        .or_else(|| first_animation_bool(prepared, "looping"))
        .unwrap_or(true);

    Some(SpriteSheet {
        columns,
        rows,
        frame_count: prepared
            .metadata
            .get("frame_count")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(columns.saturating_mul(rows))
            .max(1),
        frame_size: Vec2::new(frame_width, frame_height),
        fps,
        looping,
    })
}

pub fn resolve_sprite_sheet_for_command(
    asset_catalog: &AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    let explicit_sheet = command.sheet.as_ref().map(|sheet| SpriteSheet {
        columns: sheet.columns,
        rows: sheet.rows,
        frame_count: sheet.frame_count,
        frame_size: sheet.frame_size,
        fps: sheet.fps,
        looping: sheet.looping,
    });

    let base_sheet = explicit_sheet.or_else(|| {
        asset_catalog
            .prepared_asset(&command.texture)
            .and_then(|prepared| infer_sprite_sheet_from_prepared_asset(&prepared))
    })?;

    Some(apply_scene_animation_override(
        base_sheet,
        command.animation.as_ref(),
    ))
}

pub(crate) fn apply_animation_override(
    mut sheet: SpriteSheet,
    override_: Option<SpriteAnimationOverride>,
) -> SpriteSheet {
    let Some(override_) = override_ else {
        return sheet;
    };

    if let Some(fps) = override_.fps {
        sheet.fps = fps.max(0.0);
    }
    if let Some(looping) = override_.looping {
        sheet.looping = looping;
    }
    sheet
}

fn apply_scene_animation_override(
    mut sheet: SpriteSheet,
    animation: Option<&SpriteAnimation2dSceneOverride>,
) -> SpriteSheet {
    let Some(animation) = animation else {
        return sheet;
    };

    if let Some(fps) = animation.fps {
        sheet.fps = fps.max(0.0);
    }
    if let Some(looping) = animation.looping {
        sheet.looping = looping;
    }
    sheet
}

fn first_animation_f32(prepared: &PreparedAsset, field: &str) -> Option<f32> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<f32>().ok())
            .flatten()
    })
}

fn first_animation_bool(prepared: &PreparedAsset, field: &str) -> Option<bool> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<bool>().ok())
            .flatten()
    })
}
