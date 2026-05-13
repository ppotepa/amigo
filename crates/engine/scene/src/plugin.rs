use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

use crate::*;

pub struct ScenePlugin;

impl RuntimePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "amigo-scene"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SceneService::default())?;
        registry.register(HydratedSceneState::default())?;
        registry.register(SceneTransitionService::default())?;
        registry.register(EntityPoolSceneService::default())?;
        registry.register(LifetimeSceneService::default())?;
        registry.register(ActivationSetSceneService::default())?;
        registry.register(SceneCommandQueue::default())?;
        registry.register(SceneEventQueue::default())?;

        let scene_handlers = registry.required::<RuntimeSceneCommandHandlerRegistry>()?;
        register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            SceneLifecycleRuntimeSceneCommandHandler,
        );
        register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            SceneActivationRuntimeSceneCommandHandler,
        );
        register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            ScenePostFx2dRuntimeSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            SceneScriptCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "lifetime",
            move |runtime| tick_lifetimes(runtime, 1.0 / 60.0),
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "scene_transition",
            move |runtime| tick_scene_transitions(runtime, 1.0 / 60.0),
        );
        Ok(())
    }
}

