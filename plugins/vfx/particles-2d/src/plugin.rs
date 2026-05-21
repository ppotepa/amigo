use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_runtime_control::RuntimeControlService;
use std::sync::Arc;

use crate::model::{PARTICLES_2D_CAPABILITY, PARTICLES_2D_PLUGIN_LABEL};
use crate::service::{Particle2dSceneService, ParticlePreset2dService};

pub const PLUGIN_ID: &str = "amigo.vfx.particles-2d";

pub struct Particle2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Particle2dPlugin;

impl RuntimePlugin for Particle2dPlugin {
    fn name(&self) -> &'static str {
        PARTICLES_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Particle2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, crate::Particle2dSceneResetHandler)?;
        registry.register(ParticlePreset2dService::default())?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_particle_2d_render_extractor_id(render_extractors.as_ref());
        }
        if !registry.has::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>() {
            registry.register(
                amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry::default(),
            )?;
        }
        if let Some(editor_apply) =
            registry.resolve::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>()
        {
            editor_apply.register(crate::Particle2dEditorRuntimeApplyProvider);
        }
        if let (Some(control), Some(particles)) = (
            registry.resolve::<RuntimeControlService>(),
            registry.resolve::<Particle2dSceneService>(),
        ) {
            control.register_provider(Arc::new(
                crate::service::ParticleEmitter2dControlProvider::new(particles),
            ));
        }
        registry.register(Particle2dDomainInfo {
            crate_name: "amigo-particles-2d-plugin",
            capability: PARTICLES_2D_CAPABILITY,
        })?;
        register_domain_plugin(
            registry,
            PARTICLES_2D_PLUGIN_LABEL,
            &[PARTICLES_2D_CAPABILITY],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Particles2dSceneCommandHandler,
        );
        if !registry.has::<amigo_devtools::RuntimeConsoleCommandRegistry>() {
            registry.register(amigo_devtools::RuntimeConsoleCommandRegistry::default())?;
        }
        amigo_devtools::register_runtime_console_command_handler(
            registry
                .required::<amigo_devtools::RuntimeConsoleCommandRegistry>()?
                .as_ref(),
            crate::devtools_console::ParticlesConsoleCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "particles_2d",
            move |runtime| {
                let dt = amigo_session::simulation_delta_seconds(runtime);
                crate::tick_particles_2d_world(runtime, dt)
            },
        );
        Ok(())
    }
}
