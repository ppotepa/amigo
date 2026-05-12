use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneEventPipelineCommandHandler;

impl SceneCommandHandler for SceneEventPipelineCommandHandler {
    fn name(&self) -> &'static str {
        "scene-event-pipeline"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_event_pipeline::can_handle_event_pipeline_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_event_pipeline::handle_event_pipeline_scene_command(
            amigo_event_pipeline::EventPipelineSceneCommandContext {
                scene_service: ctx.scene_service,
                event_pipeline_service: ctx.event_pipeline_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;
        ctx.dev_console_state.write_line(format!(
            "queued event pipeline `{}` from mod `{}`",
            outcome.id, outcome.source_mod
        ));
        Ok(())
    }
}
