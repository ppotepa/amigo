use amigo_scene::SceneCommand;

use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneScriptComponentCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneScriptComponentCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-script-component"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_scripting_rhai::can_handle_rhai_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let mod_catalog = required::<ModCatalog>(ctx.runtime)?;
        let script_runtime = required::<ScriptRuntimeService>(ctx.runtime)?;
        let outcome = amigo_scripting_rhai::handle_rhai_scene_command(
            amigo_scripting_rhai::RhaiSceneCommandContext {
                mod_catalog: mod_catalog.as_ref(),
                script_runtime: script_runtime.as_ref(),
                scene_service: ctx.scene_service,
                scene_event_queue: ctx.scene_event_queue,
                script_component_service: ctx.script_component_service,
            },
            command,
        )?;
        ctx.dev_console_state.write_line(format!(
            "queued script component `{}` from mod `{}`",
            outcome.entity_name, outcome.source_mod
        ));
        Ok(())
    }
}


