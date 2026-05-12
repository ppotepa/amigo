use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_2d_layered_image::LayeredImageAssetSource;
use amigo_scene::LightMap2dSourceSceneCommand;

pub(crate) struct SceneLighting2dCommandHandler;

impl SceneCommandHandler for SceneLighting2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-lighting-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_lighting::can_handle_lighting_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        if let SceneCommand::QueueLightMap2dSource { command } = &command {
            warn_lightmap_source_issues(ctx, command);
        }

        let outcome = amigo_2d_lighting::handle_lighting_scene_command(
            amigo_2d_lighting::LightingSceneCommandContext {
                scene_service: ctx.scene_service,
                global_light2d_scene_service: ctx.global_light2d_scene_service,
                lightmap2d_scene_service: ctx.lightmap2d_scene_service,
                light_group2d_scene_service: ctx.light_group2d_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        match outcome {
            amigo_2d_lighting::LightingSceneCommandOutcome::GlobalLight {
                id, entity_name, ..
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued global 2d light `{}` on entity `{}`",
                    id, entity_name
                ));
            }
            amigo_2d_lighting::LightingSceneCommandOutcome::LightMapSource {
                id,
                entity_name,
                channel_count,
                ..
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued 2d lightmap source `{}` on entity `{}` with {} channels",
                    id, entity_name, channel_count
                ));
            }
            amigo_2d_lighting::LightingSceneCommandOutcome::LightGroup { id, .. } => {
                ctx.dev_console_state
                    .write_line(format!("queued 2d light group `{id}`"));
            }
        }

        Ok(())
    }
}

fn warn_lightmap_source_issues(
    ctx: &AppSceneCommandContext<'_>,
    command: &LightMap2dSourceSceneCommand,
) {
    if command.id.trim().is_empty() {
        ctx.dev_console_state.write_line(format!(
            "2d lightmap source on entity `{}` has an empty id",
            command.entity_name
        ));
    }

    if command.source.entity_name.trim().is_empty() {
        ctx.dev_console_state.write_line(format!(
            "2d lightmap source `{}` has an empty source entity",
            command.id
        ));
        return;
    }

    if command.channels.is_empty() {
        ctx.dev_console_state.write_line(format!(
            "2d lightmap source `{}` has no channels",
            command.id
        ));
    }

    for channel in &command.channels {
        if channel.id.trim().is_empty() {
            ctx.dev_console_state.write_line(format!(
                "2d lightmap source `{}` has a channel with an empty id",
                command.id
            ));
        }
        if channel.layers.is_empty() {
            ctx.dev_console_state.write_line(format!(
                "2d lightmap source `{}` channel `{}` has no layers",
                command.id, channel.id
            ));
        }
    }

    let layered_image_commands = ctx.layered_image_scene_service.commands();
    let Some(layered_image) = layered_image_commands
        .iter()
        .find(|item| item.entity_name == command.source.entity_name)
    else {
        ctx.dev_console_state.write_line(format!(
            "2d lightmap source `{}` references missing layered image entity `{}`",
            command.id, command.source.entity_name
        ));
        return;
    };

    let Some(asset) = ctx
        .asset_catalog
        .layered_image_asset(&layered_image.image.asset)
    else {
        ctx.dev_console_state.write_line(format!(
            "2d lightmap source `{}` could not resolve layered image asset `{}`",
            command.id,
            layered_image.image.asset.as_str()
        ));
        return;
    };

    for channel in &command.channels {
        for layer_id in &channel.layers {
            if !asset.layers.iter().any(|layer| layer.id == *layer_id) {
                ctx.dev_console_state.write_line(format!(
                    "2d lightmap source `{}` channel `{}` references missing layer `{}` in asset `{}`",
                    command.id,
                    channel.id,
                    layer_id,
                    asset.key.as_str()
                ));
            }
        }
    }
}
