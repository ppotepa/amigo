pub struct BehaviorPlugin;

impl RuntimePlugin for BehaviorPlugin {
    fn name(&self) -> &'static str {
        "amigo-behavior"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(BehaviorSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, BehaviorSceneResetHandler)?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::BehaviorSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                amigo_scene::BEHAVIOR_PLUGIN_SCENE_COMMAND_TYPE,
                std::sync::Arc::new(crate::scene_command::BehaviorSceneCommandHandler),
            );
        }
        registry
            .required::<amigo_runtime::SystemRegistry>()?
            .register_fn(
                amigo_runtime::SystemPhase::Update,
                "behavior",
                move |runtime| {
                    let dt = amigo_session::simulation_delta_seconds(runtime);
                    crate::tick_behaviors(runtime, dt)
                },
            );
        Ok(())
    }
}
