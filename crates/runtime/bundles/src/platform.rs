use amigo_core::{AmigoResult, LaunchSelection};
use amigo_input_actions::InputActionPlugin;
use amigo_input_winit::WinitInputPlugin;
use amigo_render_wgpu::WgpuRenderPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;
use amigo_window_winit::WinitWindowPlugin;

pub struct PlatformRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    pub launch_selection: LaunchSelection,
    pub app_host_plugins: F,
}

impl<F> PluginBundle for PlatformRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    fn name(&self) -> &'static str {
        "amigo-platform-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        let builder = builder
            .with_plugin(WinitWindowPlugin::default())?
            .with_plugin(WinitInputPlugin)?
            .with_plugin(InputActionPlugin)?
            .with_plugin(WgpuRenderPlugin)?;
        (self.app_host_plugins)(builder, self.launch_selection)
    }
}

pub fn register_platform_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_input_actions::register_input_actions_runtime_capabilities(session);
}

