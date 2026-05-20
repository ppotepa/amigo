use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::Particle2dSceneService;

pub struct Particle2dSceneResetHandler;

impl SceneResetHandler for Particle2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.vfx.particles-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<Particle2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
