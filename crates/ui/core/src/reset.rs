use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneResetHandler;

use crate::{UiModelBindingService, UiSceneService, UiStateService, UiThemeService};

pub struct UiSceneResetHandler;

impl SceneResetHandler for UiSceneResetHandler {
    fn name(&self) -> &'static str {
        "ui"
    }

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()> {
        if let Some(service) = runtime.resolve::<UiSceneService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<UiStateService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<UiModelBindingService>() {
            service.clear();
        }
        if let Some(service) = runtime.resolve::<UiThemeService>() {
            service.clear();
        }
        Ok(())
    }
}
