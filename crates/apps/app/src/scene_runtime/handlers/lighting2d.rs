use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use amigo_2d_layered_image::LayeredImageAssetSource;

pub(crate) struct SceneLighting2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneLighting2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-lighting-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_lighting::can_handle_lighting_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_lighting::handle_lighting_scene_command(
            amigo_2d_lighting::LightingSceneCommandContext {
                scene_service: ctx.scene_service,
                global_light2d_scene_service: ctx.global_light2d_scene_service,
                lightmap2d_scene_service: ctx.lightmap2d_scene_service,
                light_group2d_scene_service: ctx.light_group2d_scene_service,
                scene_event_queue: ctx.scene_event_queue,
                resolve_lightmap_source_layers: &|entity_name| {
                    let layered_image_commands = ctx.layered_image_scene_service.commands();
                    let layered_image = layered_image_commands
                        .iter()
                        .find(|item| item.entity_name == entity_name)?;
                    let asset = ctx
                        .asset_catalog
                        .layered_image_asset(&layered_image.image.asset)?;
                    Some(amigo_2d_lighting::LightingLayeredImageSourceAsset {
                        key: asset.key.as_str().to_owned(),
                        layer_ids: asset.layers.iter().map(|layer| layer.id.clone()).collect(),
                    })
                },
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
                warnings,
                ..
            } => {
                for warning in warnings {
                    ctx.dev_console_state.write_line(warning);
                }
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


