use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::VectorSceneService;

pub struct Vector2dSceneResetHandler;

impl SceneResetHandler for Vector2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.gfx.vector-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<VectorSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
