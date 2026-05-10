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
        matches!(
            command,
            SceneCommand::QueueGlobalLight2d { .. }
                | SceneCommand::QueueLightMap2dSource { .. }
                | SceneCommand::QueueLightGroup2d { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueGlobalLight2d { command } => {
                let entity = amigo_2d_lighting::queue_global_light_2d_scene_command(
                    ctx.scene_service,
                    ctx.global_light2d_scene_service,
                    &command,
                );
                ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                    entity_id: entity.raw(),
                    name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued global 2d light `{}` on entity `{}`",
                    command.id, command.entity_name
                ));
                Ok(())
            }
            SceneCommand::QueueLightMap2dSource { command } => {
                warn_lightmap_source_issues(ctx, &command);
                let entity = amigo_2d_lighting::queue_lightmap_2d_source_scene_command(
                    ctx.scene_service,
                    ctx.lightmap2d_scene_service,
                    &command,
                );
                ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                    entity_id: entity.raw(),
                    name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d lightmap source `{}` on entity `{}` with {} channels",
                    command.id,
                    command.entity_name,
                    command.channels.len()
                ));
                Ok(())
            }
            SceneCommand::QueueLightGroup2d { command } => {
                let id = command.id.clone();
                amigo_2d_lighting::queue_light_group_2d_scene_command(
                    ctx.light_group2d_scene_service,
                    command,
                );
                ctx.dev_console_state
                    .write_line(format!("queued 2d light group `{id}`"));
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
