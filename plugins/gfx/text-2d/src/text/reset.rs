use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::Text2dSceneService;

pub struct Text2dSceneResetHandler;

impl SceneResetHandler for Text2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.gfx.text-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Text2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
