use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use super::TileMap2dSceneService;

pub struct TileMap2dSceneResetHandler;

impl SceneResetHandler for TileMap2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.gfx.tilemap-2d"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<TileMap2dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
