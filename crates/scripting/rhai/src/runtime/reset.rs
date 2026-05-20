pub struct RhaiScriptingSceneResetHandler;

impl amigo_scene::SceneResetHandler for RhaiScriptingSceneResetHandler {
    fn name(&self) -> &'static str {
        "scripting_rhai"
    }

    fn reset_scene(&self, runtime: &amigo_runtime::Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<ScriptComponentService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<ScriptTraceService>() {
            service.clear();
        }
        Ok(())
    }
}
