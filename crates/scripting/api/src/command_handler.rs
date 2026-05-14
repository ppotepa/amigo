use std::sync::Arc;

use amigo_core::AmigoResult;
use amigo_runtime::{HandlerRegistry, Runtime};

use crate::ScriptCommand;

pub trait RuntimeScriptCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &ScriptCommand) -> bool;
    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()>;
}

pub type RuntimeScriptCommandHandlerRegistry = HandlerRegistry<dyn RuntimeScriptCommandHandler>;

pub fn register_runtime_script_command_handler<H>(
    registry: &RuntimeScriptCommandHandlerRegistry,
    handler: H,
) where
    H: RuntimeScriptCommandHandler + 'static,
{
    registry.register_arc(Arc::new(handler));
}
