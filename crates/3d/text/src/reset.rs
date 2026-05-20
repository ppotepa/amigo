use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::Text3dSceneService;

pub struct Text3dSceneResetHandler;

impl SceneResetHandler for Text3dSceneResetHandler {
    fn name(&self) -> &'static str {
        "text_3d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Text3dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
