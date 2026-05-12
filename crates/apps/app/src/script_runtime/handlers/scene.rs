use super::super::super::*;
use super::super::AppScriptCommandContext;
use amigo_session::ScriptCommandHandler;

pub(super) struct SceneScriptCommandHandler;

impl<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
    for SceneScriptCommandHandler
{
    fn name(&self) -> &'static str {
        "scene"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "scene")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
        let outcome = amigo_scene::handle_scene_script_command(
            amigo_scene::SceneScriptCommandContext {
                scene_command_queue: ctx.scene_command_queue,
            },
            command.clone(),
        );

        if outcome == amigo_scene::SceneScriptCommandOutcome::Unhandled {
            ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            ));
        }
    }
}



