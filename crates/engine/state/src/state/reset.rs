pub struct StateSceneResetHandler;

impl amigo_scene::SceneResetHandler for StateSceneResetHandler {
    fn name(&self) -> &'static str {
        "state"
    }

    fn reset_scene(&self, runtime: &amigo_runtime::Runtime) -> amigo_core::AmigoResult<()> {
        if let Some(service) = runtime.resolve::<SceneStateService>() {
            service.clear_scene_defaults();
        }
        if let Some(service) = runtime.resolve::<SceneTimerService>() {
            service.reset_scene();
        }
        Ok(())
    }
}
