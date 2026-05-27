use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use super::service::SpriteSceneService;

#[derive(Debug, Clone)]
pub struct SpriteDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct SpritePlugin;

impl RuntimePlugin for SpritePlugin {
    fn name(&self) -> &'static str {
        "amigo-sprite-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SpriteSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::Sprite2dSceneResetHandler,
        )?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::Sprite2dSceneComponentSpec,
        >(registry)?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_sprite_2d_render_extractor_id(render_extractors.as_ref());
        }
        registry.register(SpriteDomainInfo {
            crate_name: "amigo-sprite-2d-plugin",
            capability: "rendering_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-sprite-2d-plugin",
            &["rendering_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::Sprite2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                "amigo.gfx.sprite-2d.scene-command.Sprite2D",
                std::sync::Arc::new(super::scene_command::Sprite2dSceneCommandHandler),
            );
        }
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::Sprite2dScriptCommandHandler,
        );
        Ok(())
    }
}
