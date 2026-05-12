use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneInputActionsCommandHandler;

impl SceneCommandHandler for SceneInputActionsCommandHandler {
    fn name(&self) -> &'static str {
        "scene-input-actions"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_input_actions::can_handle_input_actions_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_input_actions::handle_input_actions_scene_command(
            amigo_input_actions::InputActionsSceneCommandContext {
                scene_service: ctx.scene_service,
                input_action_service: ctx.input_action_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;
        ctx.dev_console_state.write_line(format!(
            "queued input action map `{}` from mod `{}` with {} actions",
            outcome.id, outcome.source_mod, outcome.action_count
        ));
        Ok(())
    }
}
