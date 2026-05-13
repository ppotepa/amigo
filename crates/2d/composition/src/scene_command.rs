use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, format_scene_command};

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
        SceneCommand::QueueRenderLayer2d { .. } | SceneCommand::QueueLightRoute2d { .. }
    )
}

pub fn handle_composition_scene_command(
    ctx: CompositionSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<CompositionSceneCommandOutcome> {
    match command {
        SceneCommand::QueueRenderLayer2d { command } => {
            let id = command.id.clone();
            let source_mod = command.source_mod.clone();
            ctx.render_layer2d_scene_service.queue(command.into());
            Ok(CompositionSceneCommandOutcome::RenderLayer { id, source_mod })
        }
        SceneCommand::QueueLightRoute2d { command } => {
            let receiver_layer = command.receiver_layer.clone();
            let source_mod = command.source_mod.clone();
            ctx.light_route2d_scene_service.queue(command.into());
            Ok(CompositionSceneCommandOutcome::LightRoute {
                receiver_layer,
                source_mod,
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

