use amigo_core::{AmigoResult, LaunchSelection};
use amigo_modding::ModdingPlugin;
use amigo_runtime::RuntimeBuilder;
use amigo_session::{
    RenderSessionService, SceneSessionService, SchedulerSessionService, ScriptSessionService,
};

use crate::FullRuntimeBundle;

pub fn compose_game_runtime<F>(
    launch_selection: LaunchSelection,
    app_host_plugins: F,
    modding_plugin: ModdingPlugin,
    enable_devtools: bool,
) -> AmigoResult<RuntimeBuilder>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    RuntimeBuilder::default()
        .with_service(SceneSessionService::new())?
        .with_service(RenderSessionService::new())?
        .with_service(SchedulerSessionService::new())?
        .with_service(ScriptSessionService::new())?
        .with_bundle(FullRuntimeBundle {
            launch_selection,
            app_host_plugins,
            modding_plugin,
            enable_devtools,
        })
}
