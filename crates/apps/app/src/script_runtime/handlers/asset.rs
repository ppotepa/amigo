use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct AssetScriptCommandHandler;

impl ScriptCommandHandler for AssetScriptCommandHandler {
    fn name(&self) -> &'static str {
        "asset"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "asset")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("reload", [asset_key]) => {
                crate::orchestration::request_asset_reload(
                    ctx.asset_catalog,
                    asset_key,
                    AssetLoadPriority::Immediate,
                    ctx.dev_console_state,
                );
                ctx.script_event_queue.publish(ScriptEvent::new(
                    "asset.reload-requested",
                    vec![asset_key.clone()],
                ));
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
