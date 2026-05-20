pub struct BehaviorSceneResetHandler;

impl amigo_scene::SceneResetHandler for BehaviorSceneResetHandler {
    fn name(&self) -> &'static str {
        "behavior"
    }

    fn reset_scene(&self, runtime: &amigo_runtime::Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<BehaviorSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
