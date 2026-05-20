pub struct AudioOutputSceneResetHandler;

impl amigo_scene::SceneResetHandler for AudioOutputSceneResetHandler {
    fn name(&self) -> &'static str {
        "audio_output"
    }

    fn reset_scene(&self, runtime: &amigo_runtime::Runtime) -> amigo_core::AmigoResult<()> {
        if let Some(service) = runtime.resolve::<AudioOutputBackendService>() {
            service.clear_buffer();
        }
        Ok(())
    }
}
