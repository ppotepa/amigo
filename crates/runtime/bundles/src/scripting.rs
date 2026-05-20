use amigo_core::AmigoResult;
use amigo_modding::ModdingPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_session::RuntimeSession;

pub use amigo_scripting_rhai::tick_script_components;

pub struct ScriptingRuntimeBundle {
    pub modding_plugin: ModdingPlugin,
}

impl PluginBundle for ScriptingRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-modding-and-scripting-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(self.modding_plugin)?
            .with_plugin(RhaiScriptingPlugin)
    }
}

pub fn register_modding_and_scripting_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_scripting_rhai::register_rhai_runtime_capabilities(session);
}
