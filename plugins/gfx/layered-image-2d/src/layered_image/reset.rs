use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::LayeredImageSceneService;

pub struct LayeredImage2dSceneResetHandler;

impl SceneResetHandler for LayeredImage2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.gfx.layered-image-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<LayeredImageSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
