use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::{CameraController3dSceneService, CameraFollow2dSceneService, Parallax2dSceneService};

pub struct CameraCoreSceneResetHandler;

impl SceneResetHandler for CameraCoreSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.camera.camera-core"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<CameraFollow2dSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<Parallax2dSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<CameraController3dSceneService>() {
            service.clear();
        }
        Ok(())
    }
}
