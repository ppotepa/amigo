use amigo_assets::AssetsPlugin;
use amigo_behavior::BehaviorPlugin;
use amigo_camera_core_plugin::CameraPlugin;
use amigo_core::AmigoResult;
use amigo_event_pipeline::EventPipelinePlugin;
use amigo_file_watch_notify::NotifyFileWatchPlugin;
use amigo_hot_reload::HotReloadPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder, RuntimePlugin, ServiceRegistry};
use amigo_scene::ScenePlugin;
use amigo_session::RuntimeSession;
use amigo_state::StatePlugin;

pub use amigo_event_pipeline::{EventPipelineService, EventPipelineStep};

pub struct CoreRuntimeBundle;

impl PluginBundle for CoreRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-core-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(SystemRegistryPlugin)?
            .with_plugin(SceneCommandRegistryPlugin)?
            .with_plugin(ScriptCommandRegistryPlugin)?
            .with_plugin(RenderExtractorIdRegistryPlugin)?
            .with_plugin(WgpuRenderExtractorBridgeRegistryPlugin)?
            .with_plugin(AssetsPlugin)?
            .with_plugin(HotReloadPlugin)?
            .with_plugin(NotifyFileWatchPlugin)?
            .with_plugin(ScenePlugin)?
            .with_plugin(CameraPlugin)?
            .with_plugin(StatePlugin)?
            .with_plugin(BehaviorPlugin)?
            .with_plugin(EventPipelinePlugin)
    }
}

struct ScriptCommandRegistryPlugin;

struct SceneCommandRegistryPlugin;

struct SystemRegistryPlugin;

struct RenderExtractorIdRegistryPlugin;

struct WgpuRenderExtractorBridgeRegistryPlugin;

impl RuntimePlugin for SystemRegistryPlugin {
    fn name(&self) -> &'static str {
        "amigo-system-registry"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(amigo_runtime::SystemRegistry::default())?;
        Ok(())
    }
}

impl RuntimePlugin for SceneCommandRegistryPlugin {
    fn name(&self) -> &'static str {
        "amigo-scene-command-registry"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(amigo_scene::RuntimeSceneCommandHandlerRegistry::new())?;
        Ok(())
    }
}

impl RuntimePlugin for ScriptCommandRegistryPlugin {
    fn name(&self) -> &'static str {
        "amigo-script-command-registry"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(amigo_scripting_api::RuntimeScriptCommandHandlerRegistry::new())?;
        Ok(())
    }
}

impl RuntimePlugin for RenderExtractorIdRegistryPlugin {
    fn name(&self) -> &'static str {
        "amigo-render-extractor-id-registry"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(amigo_render_api::RuntimeRenderExtractorIdRegistry::default())?;
        Ok(())
    }
}

impl RuntimePlugin for WgpuRenderExtractorBridgeRegistryPlugin {
    fn name(&self) -> &'static str {
        "amigo-wgpu-render-extractor-bridge-registry"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(
            crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry::default(),
        )?;
        Ok(())
    }
}

pub fn register_core_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_assets::register_assets_runtime_capabilities(session);
    amigo_scene::register_scene_runtime_capabilities(session);
    amigo_camera_core_plugin::register_camera_runtime_capabilities(session);
    amigo_behavior::register_behavior_runtime_capabilities(session);
    amigo_event_pipeline::register_event_pipeline_runtime_capabilities(session);
    amigo_render_api::register_render_runtime_capabilities(session);
    amigo_session::register_session_runtime_capabilities(session);
}
