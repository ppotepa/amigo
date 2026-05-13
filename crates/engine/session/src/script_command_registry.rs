use std::sync::Arc;

use amigo_runtime::HandlerRegistry;

pub trait ScriptCommandHandler<Ctx, Command, Output>: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &Command) -> bool;
    fn handle(&self, ctx: &Ctx, command: Command) -> Output;
}

pub type ScriptCommandHandlerObject<Ctx, Command, Output> =
    dyn ScriptCommandHandler<Ctx, Command, Output>;

pub type ScriptCommandRegistry<Ctx, Command, Output> =
    HandlerRegistry<ScriptCommandHandlerObject<Ctx, Command, Output>>;

pub fn register_script_command_handler<Ctx, Command, Output, H>(
    registry: &mut ScriptCommandRegistry<Ctx, Command, Output>,
    handler: H,
) where
    H: ScriptCommandHandler<Ctx, Command, Output> + 'static,
{
    registry.register_arc(Arc::new(handler));
}

