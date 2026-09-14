use amigo_core::{AmigoResult, LaunchSelection};
use amigo_modding::ModdingPlugin;
use amigo_runtime::{RuntimeBuilder, RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_session::{
    RenderSessionService, RuntimeLoadingService, SceneSessionService, SchedulerSessionService,
    ScriptSessionService,
};

use crate::FullRuntimeBundle;

/// Small, engine-wide budget for cooperative loading jobs. Individual jobs
/// account their own units and may yield before consuming the full budget.
const RUNTIME_LOADING_WORK_BUDGET: u64 = 4;

struct RuntimeLoadingPlugin;

impl RuntimePlugin for RuntimeLoadingPlugin {
    fn name(&self) -> &'static str {
        "amigo-runtime-loading"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PreUpdate,
            "runtime_loading",
            |runtime| {
                runtime
                    .required::<RuntimeLoadingService>()?
                    .run_frame(RUNTIME_LOADING_WORK_BUDGET);
                Ok(())
            },
        );
        Ok(())
    }
}

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
        .with_service(RuntimeLoadingService::new())?
        .with_service(RenderSessionService::new())?
        .with_service(SchedulerSessionService::new())?
        .with_service(ScriptSessionService::new())?
        .with_bundle(FullRuntimeBundle {
            launch_selection,
            app_host_plugins,
            modding_plugin,
            enable_devtools,
        })?
        .with_plugin(RuntimeLoadingPlugin)
}
