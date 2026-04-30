use std::sync::Arc;

use super::super::*;
use super::context::AppSceneCommandContext;
use super::handlers::register_builtin_scene_command_handlers;
use amigo_runtime::{HandlerRegistry, RoutedHandler};

pub(crate) trait SceneCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &SceneCommand) -> bool;
    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()>;
}

struct SceneCommandHandlerAdapter<H>(H);

impl<H> RoutedHandler<AppSceneCommandContext<'_>, SceneCommand, AmigoResult<()>>
    for SceneCommandHandlerAdapter<H>
where
    H: SceneCommandHandler,
{
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        self.0.can_handle(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        self.0.handle(ctx, command)
    }
}

pub(crate) type SceneCommandHandlerObject =
    dyn for<'a> RoutedHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>;

pub(crate) type SceneCommandHandlerRegistry = HandlerRegistry<SceneCommandHandlerObject>;

pub(crate) fn register_scene_command_handler<H>(
    registry: &mut SceneCommandHandlerRegistry,
    handler: H,
) where
    H: SceneCommandHandler + 'static,
{
    registry.register_arc(Arc::new(SceneCommandHandlerAdapter(handler)));
}

pub(crate) struct SceneCommandRuntimePlugin;

impl RuntimePlugin for SceneCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-scene-command-registry"
    }

    fn register(&self, services: &mut ServiceRegistry) -> AmigoResult<()> {
        let mut registry = SceneCommandHandlerRegistry::new();
        register_builtin_scene_command_handlers(&mut registry);
        services.register(registry)
    }
}
