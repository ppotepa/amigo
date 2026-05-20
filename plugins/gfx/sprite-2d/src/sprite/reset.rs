use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::SpriteSceneService;

pub struct Sprite2dSceneResetHandler;

impl SceneResetHandler for Sprite2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.gfx.sprite-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<SpriteSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
