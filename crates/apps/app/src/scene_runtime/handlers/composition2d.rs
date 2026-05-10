use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneComposition2dCommandHandler;

impl SceneCommandHandler for SceneComposition2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-composition-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueRenderLayer2d { .. } | SceneCommand::QueueLightRoute2d { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueRenderLayer2d { command } => {
                ctx.render_layer2d_scene_service.queue(command.into());
                Ok(())
            }
            SceneCommand::QueueLightRoute2d { command } => {
                ctx.light_route2d_scene_service.queue(command.into());
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
