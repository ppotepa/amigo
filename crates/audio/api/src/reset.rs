use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::{AudioSceneService, AudioStateService};

pub struct AudioSceneResetHandler;

impl SceneResetHandler for AudioSceneResetHandler {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<AudioSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<AudioStateService>() {
            service.clear();
        }
        Ok(())
    }
}
