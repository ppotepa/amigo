use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::Physics2dSceneService;

pub struct Physics2dSceneResetHandler;

impl SceneResetHandler for Physics2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "physics_2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Physics2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
