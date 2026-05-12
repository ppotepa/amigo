use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneBehaviorCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneBehaviorCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-behavior"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_behavior::can_handle_behavior_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_behavior::handle_behavior_scene_command(
            amigo_behavior::BehaviorSceneCommandContext {
                scene_service: ctx.scene_service,
                scene_event_queue: ctx.scene_event_queue,
                behavior_scene_service: ctx.behavior_scene_service,
            },
            command,
        )?;
        ctx.dev_console_state.write_line(format!(
            "queued behavior `{}` from mod `{}`",
            outcome.entity_name, outcome.source_mod
        ));
        Ok(())
    }
}


