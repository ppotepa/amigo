use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::Motion2dSceneService;

pub struct Motion2dSceneResetHandler;

impl SceneResetHandler for Motion2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.camera.shutter-motion"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Motion2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
