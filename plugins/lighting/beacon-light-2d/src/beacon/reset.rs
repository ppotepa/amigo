use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::BeaconLight2dSceneService;

pub struct BeaconLight2dSceneResetHandler;

impl SceneResetHandler for BeaconLight2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<BeaconLight2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
