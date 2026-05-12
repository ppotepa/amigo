pub struct BehaviorPlugin;

impl RuntimePlugin for BehaviorPlugin {
    fn name(&self) -> &'static str {
        "amigo-behavior"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(BehaviorSceneService::default())?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::BehaviorSceneCommandHandler,
        );
        Ok(())
    }
}
