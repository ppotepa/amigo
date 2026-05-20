use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::{LightRoute2dSceneService, RenderLayer2dSceneService};

pub struct Composition2dSceneResetHandler;

impl SceneResetHandler for Composition2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.2d.composition"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<RenderLayer2dSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<LightRoute2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
