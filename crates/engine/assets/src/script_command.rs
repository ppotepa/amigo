use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{
    RuntimeScriptCommandHandler, ScriptCommand, ScriptEvent, ScriptEventQueue,
};
use crate::{AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest};

pub struct AssetScriptCommandContext<'a> {
    pub asset_catalog: &'a AssetCatalog,
    pub script_event_queue: &'a ScriptEventQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetScriptCommandOutcome {
    ReloadRequested { asset_key: String },
    Unhandled,
}

pub fn can_handle_asset_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "asset"
}

pub fn handle_asset_script_command(
    ctx: AssetScriptCommandContext<'_>,
    command: ScriptCommand,
) -> AssetScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("reload", [asset_key]) => {
            ctx.asset_catalog
                .request_reload(AssetLoadRequest::new(
                    AssetKey::new(asset_key.clone()),
                    AssetLoadPriority::Immediate,
                ));
            ctx.script_event_queue.publish(ScriptEvent::new(
                "asset.reload-requested",
                vec![asset_key.clone()],
            ));
            AssetScriptCommandOutcome::ReloadRequested {
                asset_key: asset_key.clone(),
            }
        }
        _ => AssetScriptCommandOutcome::Unhandled,
    }
}

pub struct AssetScriptCommandHandler;

impl RuntimeScriptCommandHandler for AssetScriptCommandHandler {
    fn name(&self) -> &'static str {
        "asset"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_asset_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let asset_catalog = runtime.required::<AssetCatalog>()?;
        let script_event_queue = runtime.required::<ScriptEventQueue>()?;
        let _ = handle_asset_script_command(
            AssetScriptCommandContext {
                asset_catalog: asset_catalog.as_ref(),
                script_event_queue: script_event_queue.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
