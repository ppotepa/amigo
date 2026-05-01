pub struct StatePlugin;

impl RuntimePlugin for StatePlugin {
    fn name(&self) -> &'static str {
        "amigo-state"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        if !registry.has::<SceneStateService>() {
            registry.register(SceneStateService::default())?;
        }
        if !registry.has::<SceneTimerService>() {
            registry.register(SceneTimerService::default())?;
        }
        if !registry.has::<SessionStateService>() {
            registry.register(SessionStateService::default())?;
        }
        Ok(())
    }
}
