use crate::model::{Sprite, SpriteAnimationOverride, SpriteDrawCommand, SpriteSheet};
use crate::service::SpriteSceneService;
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_math::Vec2;
use amigo_render_api::{
    CameraOpticalResponse2d, Material2d, Material2dLighting, Material2dOptical,
    Material2dOpticalMode, RenderContributionSet, render_contribution_roles as roles,
};
use amigo_scene::{
    Material2dOpticalModeSceneCommand, Material2dSceneCommand, SceneEntityId, SceneService,
    Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
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
        render_layer: command.render_layer.clone(),
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
            visual_maps: command.visual_maps.clone(),
            frame_index: command
                .animation
                .as_ref()
                .and_then(|animation| animation.start_frame)
                .unwrap_or(0),
            frame_elapsed: 0.0,
        },
        z_index: command.z_index,
        transform: command.transform,
        material: material_from_scene_command(command.material.as_ref()),
        render_contributions: sprite_render_contributions(command),
    });
    entity
}

fn sprite_render_contributions(command: &Sprite2dSceneCommand) -> RenderContributionSet {
    let mut render_contributions =
        RenderContributionSet::from_pairs(command.render_contributions.roles.clone());
    render_contributions.merge_defaults([
        (roles::WORLD_COLOR, true),
        (roles::MATERIAL_MASK, false),
        (roles::OPTICS_REFRACT, false),
        (roles::TRANSMISSION_SOURCE, false),
        (roles::BLOOM_SOURCE, false),
        (roles::CAMERA_FX_SOURCE, false),
    ]);
    render_contributions
}

fn material_from_scene_command(material: Option<&Material2dSceneCommand>) -> Option<Material2d> {
    material.map(|material| {
        Material2d {
            optical: Material2dOptical {
                mode: match material.optical.mode {
                    Material2dOpticalModeSceneCommand::Opaque => Material2dOpticalMode::Opaque,
                    Material2dOpticalModeSceneCommand::Transmissive => {
                        Material2dOpticalMode::Transmissive
                    }
                    Material2dOpticalModeSceneCommand::Refractive => {
                        Material2dOpticalMode::Refractive
                    }
                    Material2dOpticalModeSceneCommand::Emissive => Material2dOpticalMode::Emissive,
                },
                transmission: material.optical.transmission,
                refraction_px: material.optical.refraction_px,
                distortion: material.optical.distortion,
                dispersion: material.optical.dispersion,
                roughness: material.optical.roughness,
                edge_boost: material.optical.edge_boost,
            },
            lighting: Material2dLighting {
                receives_light: material.lighting.receives_light,
                response: material.lighting.response,
            },
            camera_response: CameraOpticalResponse2d {
                enabled: material.camera_response.enabled,
                intensity: material.camera_response.intensity,
                bloom: material.camera_response.bloom,
                glare: material.camera_response.glare,
                ghosting: material.camera_response.ghosting,
                streaks: material.camera_response.streaks,
                chromatic_smear: material.camera_response.chromatic_smear,
                dirt_response: material.camera_response.dirt_response,
                halation: material.camera_response.halation,
                threshold: material.camera_response.threshold,
            },
        }
        .normalized()
    })
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
