use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::EventPipelineService;

pub struct EventPipelineSceneResetHandler;

impl SceneResetHandler for EventPipelineSceneResetHandler {
    fn name(&self) -> &'static str {
        "event_pipeline"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<EventPipelineService>() {
            service.clear();
        }
        Ok(())
    }
}
