use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::MaterialSceneService;

pub struct MaterialSceneResetHandler;

impl SceneResetHandler for MaterialSceneResetHandler {
    fn name(&self) -> &'static str {
        "material_3d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<MaterialSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
