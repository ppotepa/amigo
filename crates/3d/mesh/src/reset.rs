use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::MeshSceneService;

pub struct MeshSceneResetHandler;

impl SceneResetHandler for MeshSceneResetHandler {
    fn name(&self) -> &'static str {
        "mesh_3d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<MeshSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
