use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::Physics3dSceneService;

pub struct Physics3dSceneResetHandler;

impl SceneResetHandler for Physics3dSceneResetHandler {
    fn name(&self) -> &'static str {
        "physics_3d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Physics3dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
