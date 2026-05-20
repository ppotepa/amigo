use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{
    CompositePostFx2dRuntimeSceneCommandHandler, PostFx2dService, POST_FX_2D_CAPABILITY,
    POST_FX_2D_PLUGIN_LABEL,
};

pub const PLUGIN_ID: &str = "amigo.postfx.composite";

pub struct PostFx2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct PostFx2dPlugin;

pub struct PostFx2dConsoleCompletionProvider;

impl amigo_devtools::ConsoleCompletionProvider for PostFx2dConsoleCompletionProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.postfx.composite"
    }

    fn augment_context(
        &self,
        runtime: &amigo_runtime::Runtime,
        context: &mut amigo_devtools::ConsoleCompletionContext,
    ) {
        context.postfx_indices = runtime
            .resolve::<PostFx2dService>()
            .map(|postfx| {
                (0..postfx.frame_effect_count())
                    .map(|index| index.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    }

    fn postfx_kinds(&self, _runtime: &amigo_runtime::Runtime) -> Vec<String> {
        [
            "blur",
            "crt",
            "dirty_bloom",
            "rain_glass",
            "lens_droplets",
            "color_quantize",
            "film_noise",
            "shutter_blur",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
}

impl RuntimePlugin for PostFx2dPlugin {
    fn name(&self) -> &'static str {
        POST_FX_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(PostFx2dService::default())?;
        if !registry.has::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>() {
            registry.register(
                amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry::default(),
            )?;
        }
        if let Some(editor_apply) =
            registry.resolve::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>()
        {
            editor_apply.register(crate::CompositeEditorRuntimeApplyProvider);
        }
        amigo_scene::register_scene_reset_handler(registry, crate::PostFx2dSceneResetHandler)?;
        registry.register(PostFx2dDomainInfo {
            crate_name: "amigo-composite-plugin",
            capability: POST_FX_2D_CAPABILITY,
        })?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            CompositePostFx2dRuntimeSceneCommandHandler,
        );
        if !registry.has::<amigo_devtools::ConsoleCompletionProviderRegistry>() {
            registry.register(amigo_devtools::ConsoleCompletionProviderRegistry::default())?;
        }
        registry
            .required::<amigo_devtools::ConsoleCompletionProviderRegistry>()?
            .register(PostFx2dConsoleCompletionProvider);
        if !registry.has::<amigo_devtools::RuntimeConsoleCommandRegistry>() {
            registry.register(amigo_devtools::RuntimeConsoleCommandRegistry::default())?;
        }
        amigo_devtools::register_runtime_console_command_handler(
            registry
                .required::<amigo_devtools::RuntimeConsoleCommandRegistry>()?
                .as_ref(),
            crate::PostFxConsoleCommandHandler,
        );
        register_domain_plugin(
            registry,
            POST_FX_2D_PLUGIN_LABEL,
            &[POST_FX_2D_CAPABILITY],
            &["rendering_2d"],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
