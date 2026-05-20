use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::{GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService};

pub struct Lighting2dSceneResetHandler;

impl SceneResetHandler for Lighting2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.lighting.light-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<GlobalLight2dSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<LightMap2dSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<LightGroup2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
