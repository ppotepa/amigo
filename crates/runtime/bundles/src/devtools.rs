use amigo_core::AmigoResult;
use amigo_devtools::DevtoolsPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;

pub struct DevtoolsRuntimeBundle;

impl PluginBundle for DevtoolsRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-devtools-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder.with_plugin(DevtoolsPlugin)
    }
}

pub fn register_devtools_runtime_capabilities(_session: &mut RuntimeSession) {}

