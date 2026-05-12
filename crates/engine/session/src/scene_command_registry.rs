use std::sync::Arc;

use amigo_runtime::HandlerRegistry;

pub trait SceneCommandHandler<Ctx, Command, Output>: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &Command) -> bool;
    fn handle(&self, ctx: &Ctx, command: Command) -> Output;
}

pub type SceneCommandHandlerObject<Ctx, Command, Output> =
    dyn SceneCommandHandler<Ctx, Command, Output>;

pub type SceneCommandRegistry<Ctx, Command, Output> =
    HandlerRegistry<SceneCommandHandlerObject<Ctx, Command, Output>>;

pub fn register_scene_command_handler<Ctx, Command, Output, H>(
    registry: &mut SceneCommandRegistry<Ctx, Command, Output>,
    handler: H,
) where
    H: SceneCommandHandler<Ctx, Command, Output> + 'static,
{
    registry.register_arc(Arc::new(handler));
}
