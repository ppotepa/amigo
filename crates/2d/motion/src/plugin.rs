use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};

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
        )
    }
}
