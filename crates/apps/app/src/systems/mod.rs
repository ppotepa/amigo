//! Frame systems executed by the main application runtime.
//! They advance gameplay, UI, scripting, audio, and scene transitions after bootstrap.

use amigo_core::AmigoResult;
use amigo_runtime::{EngineTaskSystem, RuntimePlugin, ServiceRegistry};

pub(crate) struct RuntimeSystemServicesPlugin;

impl RuntimePlugin for RuntimeSystemServicesPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-runtime-system-services"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(crate::render_runtime::RenderFrameStatsService::default())?;
        registry.register(crate::render_runtime::RenderCompositionDiagnosticsService::default())?;
        registry.register(amigo_session::RuntimeSchedulingService::default())?;
        registry.register(amigo_session::RuntimeFrameClockService::default())?;
        registry.register(EngineTaskSystem::default())
    }
}
