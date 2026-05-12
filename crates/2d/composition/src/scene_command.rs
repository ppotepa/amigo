use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, format_scene_command};

use crate::{LightRoute2dSceneService, RenderLayer2dSceneService};

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
