use amigo_core::{AmigoResult, LaunchSelection};
use amigo_modding::ModdingPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};

use crate::{
    AudioRuntimeBundle, CoreRuntimeBundle, DevtoolsRuntimeBundle, PlatformRuntimeBundle,
    ScriptingRuntimeBundle, ThreeDRuntimeBundle, TwoDRuntimeBundle,
};

pub struct FullRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    pub launch_selection: LaunchSelection,
    pub app_host_plugins: F,
    pub modding_plugin: ModdingPlugin,
    pub enable_devtools: bool,
}

impl<F> PluginBundle for FullRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    fn name(&self) -> &'static str {
        "amigo-full-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        let builder = builder
            .with_bundle(CoreRuntimeBundle)?
            .with_bundle(PlatformRuntimeBundle {
                launch_selection: self.launch_selection,
                app_host_plugins: self.app_host_plugins,
            })?
            .with_bundle(TwoDRuntimeBundle)?
            .with_bundle(AudioRuntimeBundle)?
            .with_bundle(ThreeDRuntimeBundle)?
            .with_bundle(ScriptingRuntimeBundle {
                modding_plugin: self.modding_plugin,
            })?;

        if self.enable_devtools {
            builder.with_bundle(DevtoolsRuntimeBundle)
        } else {
            Ok(builder)
        }
    }
}
