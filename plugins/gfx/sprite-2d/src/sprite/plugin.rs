use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scripting_api::{ScriptBindingProviderDescriptor, ScriptBindingProviderRegistry};

use super::service::SpriteSceneService;

#[derive(Debug, Clone)]
pub struct SpriteDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct SpritePlugin;

impl RuntimePlugin for SpritePlugin {
    fn name(&self) -> &'static str { "amigo-sprite-2d-plugin" }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SpriteSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, super::reset::Sprite2dSceneResetHandler)?;
        amigo_scene::register_scene_component_plugin_spec::<crate::scene::Sprite2dSceneComponentSpec>(registry)?;

        if !registry.has::<amigo_render_api::RuntimeRenderExtractorIdRegistry>() {
            registry.register(amigo_render_api::RuntimeRenderExtractorIdRegistry::default())?;
        }
        let render_extractors = registry.required::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()?;
        crate::render::register_sprite_2d_render_extractor_id(render_extractors.as_ref());

        registry.register(SpriteDomainInfo {
            crate_name: "amigo-sprite-2d-plugin",
            capability: "gfx.sprite.2d",
        })?;

        let manifest = amigo_plugin_manifest::parse_plugin_manifest_str(include_str!("../../plugin.toml"))
            .map_err(|error| amigo_core::AmigoError::Message(format!(
                "invalid embedded sprite-2d plugin manifest: {error:?}"
            )))?;
        amigo_capabilities::register_plugin_manifest(registry, &manifest)?;

        if !registry.has::<ScriptBindingProviderRegistry>() {
            registry.register(ScriptBindingProviderRegistry::default())?;
        }
        registry.required::<ScriptBindingProviderRegistry>()?.register(
            ScriptBindingProviderDescriptor::new("amigo.gfx.sprite-2d", "sprite2d")
                .with_binding("sprite2d"),
        )?;

        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::Sprite2dSceneCommandHandler,
        );
        let plugin_scene_handlers = registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            "amigo.gfx.sprite-2d.scene-command.Sprite2D",
            std::sync::Arc::new(super::scene_command::Sprite2dSceneCommandHandler),
        );
        let script_handlers = registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::Sprite2dScriptCommandHandler,
        );
        Ok(())
    }
}
