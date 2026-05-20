use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::PostFx2dService;

pub struct PostFx2dSceneResetHandler;

impl SceneResetHandler for PostFx2dSceneResetHandler {
    fn name(&self) -> &'static str {
        "amigo.postfx.composite"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<PostFx2dService>() {
            service.clear_scoped_stacks();
            service.set_lens_certification_reports(Vec::new());
            service.set_renderer_mode("none");
        }
        Ok(())
    }
}
