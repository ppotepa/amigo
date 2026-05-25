use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{format_scene_command, SceneCommand};

use crate::{LightRoute2dSceneService, RenderLayer2dSceneService};

pub struct Composition2dSceneCommandHandler;

pub struct CompositionSceneCommandContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
    pub light_route2d_scene_service: &'a LightRoute2dSceneService,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositionSceneCommandOutcome {
    RenderLayer {
        id: String,
        source_mod: String,
    },
    LightRoute {
        receiver_layer: String,
        source_mod: String,
    },
}

pub fn can_handle_composition_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if matches!(
                command.command_type.as_str(),
                amigo_scene::RENDER_LAYER_2D_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::VISUAL2D_SPATIAL_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::LIGHT_ROUTE_2D_PLUGIN_SCENE_COMMAND_TYPE
            )
    )
}

pub fn handle_composition_scene_command(
    ctx: CompositionSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<CompositionSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::RENDER_LAYER_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(command) = command
                .payload_as::<amigo_scene::RenderLayer2dSceneCommand>()
                .cloned()
            else {
                return Err(AmigoError::Message(
                    "composition-2d render layer plugin command payload mismatch".to_owned(),
                ));
            };
            let id = command.id.clone();
            let source_mod = command.source_mod.clone();
            ctx.render_layer2d_scene_service
                .queue(crate::render_layer_2d_command_from_scene(command));
            Ok(CompositionSceneCommandOutcome::RenderLayer { id, source_mod })
        }
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::LIGHT_ROUTE_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(command) = command
                .payload_as::<amigo_scene::LightRoute2dSceneCommand>()
                .cloned()
            else {
                return Err(AmigoError::Message(
                    "composition-2d light route plugin command payload mismatch".to_owned(),
                ));
            };
            let receiver_layer = command.receiver_layer.clone();
            let source_mod = command.source_mod.clone();
            ctx.light_route2d_scene_service
                .queue(crate::light_route_2d_command_from_scene(command));
            Ok(CompositionSceneCommandOutcome::LightRoute {
                receiver_layer,
                source_mod,
            })
        }
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::VISUAL2D_SPATIAL_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(depth_space) = command
                .payload_as::<amigo_scene::DepthSpace2dSceneCommand>()
                .copied()
            else {
                return Err(AmigoError::Message(
                    "composition-2d visual2d spatial plugin command payload mismatch".to_owned(),
                ));
            };
            ctx.render_layer2d_scene_service
                .set_depth_space(depth_space.to_runtime());
            Ok(CompositionSceneCommandOutcome::RenderLayer {
                id: "visual2d.spatial".to_owned(),
                source_mod: "scene".to_owned(),
            })
        }
        _ => Err(AmigoError::Message(format!(
            "composition-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Composition2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_composition_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let render_layer2d_scene_service = runtime.required::<RenderLayer2dSceneService>()?;
        let light_route2d_scene_service = runtime.required::<LightRoute2dSceneService>()?;

        handle_composition_scene_command(
            CompositionSceneCommandContext {
                render_layer2d_scene_service: render_layer2d_scene_service.as_ref(),
                light_route2d_scene_service: light_route2d_scene_service.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
