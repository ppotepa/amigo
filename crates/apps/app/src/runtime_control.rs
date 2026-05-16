use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_runtime_control::RuntimeControlService;

pub(crate) struct RuntimeControlPlugin;

impl RuntimePlugin for RuntimeControlPlugin {
    fn name(&self) -> &'static str {
        "amigo-runtime-control"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        if !registry.has::<RuntimeControlService>() {
            registry.register(RuntimeControlService::default())?;
        }
        Ok(())
    }
}
