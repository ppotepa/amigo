use amigo_layered_image_2d_plugin::LayeredImageAssetSource;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    LightMap2dSourceSceneCommand, SceneCommand, SceneEvent, SceneEventQueue, SceneService,
    format_scene_command,
};

use super::{
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
    queue_global_light_2d_scene_command, queue_light_group_2d_scene_command,
    queue_lightmap_2d_source_scene_command,
};

pub struct Lighting2dSceneCommandHandler;

pub struct LightingSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
    pub resolve_lightmap_source_layers: &'a dyn Fn(&str) -> Option<LightingLayeredImageSourceAsset>,
}

#[derive(Debug, Clone)]
pub struct LightingLayeredImageSourceAsset {
    pub key: String,
    pub layer_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LightingSceneCommandOutcome {
    GlobalLight {
        id: String,
        entity_name: String,
        source_mod: String,
    },
    LightMapSource {
        id: String,
        entity_name: String,
        source_mod: String,
        channel_count: usize,
        warnings: Vec<String>,
    },
    LightGroup {
        id: String,
        source_mod: String,
    },
}

pub fn can_handle_lighting_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueGlobalLight2d { .. }
            | SceneCommand::QueueLightMap2dSource { .. }
            | SceneCommand::QueueLightGroup2d { .. }
    )
}

pub fn handle_lighting_scene_command(
    ctx: LightingSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<LightingSceneCommandOutcome> {
    match command {
        SceneCommand::QueueGlobalLight2d { command } => {
            let entity = queue_global_light_2d_scene_command(
                ctx.scene_service,
                ctx.global_light2d_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });
            Ok(LightingSceneCommandOutcome::GlobalLight {
                id: command.id,
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueLightMap2dSource { command } => {
            let warnings =
                collect_lightmap_source_warnings(&command, ctx.resolve_lightmap_source_layers);
            let entity = queue_lightmap_2d_source_scene_command(
                ctx.scene_service,
                ctx.lightmap2d_scene_service,
                &command,
            );
            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name: command.entity_name.clone(),
            });
            Ok(LightingSceneCommandOutcome::LightMapSource {
                id: command.id,
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                channel_count: command.channels.len(),
                warnings,
            })
        }
        SceneCommand::QueueLightGroup2d { command } => {
            let id = command.id.clone();
            let source_mod = command.source_mod.clone();
            queue_light_group_2d_scene_command(ctx.light_group2d_scene_service, command);
            Ok(LightingSceneCommandOutcome::LightGroup { id, source_mod })
        }
        _ => Err(AmigoError::Message(format!(
            "lighting-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn collect_lightmap_source_warnings(
    command: &LightMap2dSourceSceneCommand,
    resolve_lightmap_source_layers: &dyn Fn(&str) -> Option<LightingLayeredImageSourceAsset>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if command.id.trim().is_empty() {
        warnings.push(format!(
            "2d lightmap source on entity `{}` has an empty id",
            command.entity_name
        ));
    }

    if command.source.entity_name.trim().is_empty() {
        warnings.push(format!(
            "2d lightmap source `{}` has an empty source entity",
            command.id
        ));
        return warnings;
    }

    if command.channels.is_empty() {
        warnings.push(format!(
            "2d lightmap source `{}` has no channels",
            command.id
        ));
    }

    for channel in &command.channels {
        if channel.id.trim().is_empty() {
            warnings.push(format!(
                "2d lightmap source `{}` has a channel with an empty id",
                command.id
            ));
        }
        if channel.layers.is_empty() {
            warnings.push(format!(
                "2d lightmap source `{}` channel `{}` has no layers",
                command.id, channel.id
            ));
        }
    }

    let Some(asset) = resolve_lightmap_source_layers(&command.source.entity_name) else {
        warnings.push(format!(
            "2d lightmap source `{}` references missing layered image entity `{}`",
            command.id, command.source.entity_name
        ));
        return warnings;
    };

    for channel in &command.channels {
        for layer_id in &channel.layers {
            if !asset.layer_ids.iter().any(|layer| layer == layer_id) {
                warnings.push(format!(
                    "2d lightmap source `{}` channel `{}` references missing layer `{}` in asset `{}`",
                    command.id, channel.id, layer_id, asset.key
                ));
            }
        }
    }

    warnings
}

impl amigo_scene::RuntimeSceneCommandHandler for Lighting2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_lighting_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let global_light2d_scene_service = runtime.required::<GlobalLight2dSceneService>()?;
        let lightmap2d_scene_service = runtime.required::<LightMap2dSceneService>()?;
        let light_group2d_scene_service = runtime.required::<LightGroup2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        let layered_image_scene_service =
            runtime.required::<amigo_layered_image_2d_plugin::LayeredImageSceneService>()?;
        let asset_catalog = runtime.required::<amigo_assets::AssetCatalog>()?;

        handle_lighting_scene_command(
            LightingSceneCommandContext {
                scene_service: scene_service.as_ref(),
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                lightmap2d_scene_service: lightmap2d_scene_service.as_ref(),
                light_group2d_scene_service: light_group2d_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
                resolve_lightmap_source_layers: &|entity_name| {
                    let layered_image_commands = layered_image_scene_service.commands();
                    let layered_image = layered_image_commands
                        .iter()
                        .find(|item| item.entity_name == entity_name)?;
                    let asset = asset_catalog.layered_image_asset(&layered_image.image.asset)?;
                    Some(LightingLayeredImageSourceAsset {
                        key: asset.key.as_str().to_owned(),
                        layer_ids: asset.layers.iter().map(|layer| layer.id.clone()).collect(),
                    })
                },
            },
            command,
        )?;

        Ok(())
    }
}
