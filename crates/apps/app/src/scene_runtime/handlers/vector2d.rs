use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneVector2dCommandHandler;

impl SceneCommandHandler for SceneVector2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-vector-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_vector::can_handle_vector_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_vector::handle_vector_scene_command(
            amigo_2d_vector::VectorSceneCommandContext {
                scene_service: ctx.scene_service,
                vector_scene_service: ctx.vector_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        ctx.dev_console_state.write_line(format!(
            "queued 2d vector entity `{}` from mod `{}`",
            outcome.entity_name, outcome.source_mod
        ));

        Ok(())
    }
}
