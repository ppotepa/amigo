use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

use crate::service::Motion2dSceneService;

pub const CANONICAL_MOTION_2D_PLUGIN_LABEL: &str = "amigo-2d-motion";
pub const CANONICAL_MOTION_2D_CAPABILITY: &str = "motion_2d";
pub const CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL: &str = "motion_2d via amigo-2d-motion";

#[derive(Debug, Clone)]
pub struct Motion2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Motion2dPlugin;
pub const MOTION_2D_PLUGIN: Motion2dPlugin = Motion2dPlugin;

pub fn motion_2d_plugin() -> Motion2dPlugin {
    Motion2dPlugin
}

pub fn motion_runtime_plugin_report_label(plugin_name: &str) -> String {
    if plugin_name == CANONICAL_MOTION_2D_PLUGIN_LABEL {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL.to_owned()
    } else {
        plugin_name.to_owned()
    }
}

pub fn motion_2d_domain_info() -> Motion2dDomainInfo {
    Motion2dDomainInfo::canonical()
}

impl Motion2dDomainInfo {
    pub const fn canonical() -> Self {
        Self {
            crate_name: CANONICAL_MOTION_2D_PLUGIN_LABEL,
            capability: CANONICAL_MOTION_2D_CAPABILITY,
        }
    }

    pub const fn canonical_plugin_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    pub const fn runtime_report_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL
    }
}

impl Motion2dPlugin {
    pub const fn canonical_motion_plugin_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    pub const fn canonical_motion_capability(&self) -> &'static str {
        CANONICAL_MOTION_2D_CAPABILITY
    }

    pub const fn runtime_report_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL
    }
}

impl RuntimePlugin for Motion2dPlugin {
    fn name(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Motion2dSceneService::default())?;
        registry.register(Motion2dDomainInfo::canonical())?;
        register_domain_plugin(
            registry,
            CANONICAL_MOTION_2D_PLUGIN_LABEL,
            &[CANONICAL_MOTION_2D_CAPABILITY],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Motion2dSceneCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "motion_2d",
            move |runtime| crate::tick_motion_2d_world(runtime, 1.0 / 60.0),
        );
        Ok(())
    }
}
