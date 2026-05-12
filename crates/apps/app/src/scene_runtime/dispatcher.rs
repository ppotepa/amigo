use std::sync::Arc;

use super::super::*;
use super::context::AppSceneCommandContext;
use super::handlers::register_builtin_scene_command_handlers;
use amigo_runtime::HandlerRegistry;
use amigo_session::SceneCommandHandler;

pub(crate) type SceneCommandHandlerObject =
    dyn for<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>;

pub(crate) type SceneCommandHandlerRegistry =
    HandlerRegistry<SceneCommandHandlerObject>;

pub(crate) fn register_scene_command_handler<H>(
    registry: &mut SceneCommandHandlerRegistry,
    handler: H,
) where
    H: for<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
        + 'static,
{
    registry.register_arc(Arc::new(handler));
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
