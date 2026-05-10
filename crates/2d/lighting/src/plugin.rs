use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{
    GlobalLight2dSceneService, LIGHTING_2D_CAPABILITY, LIGHTING_2D_PLUGIN_LABEL,
    LightGroup2dSceneService, LightMap2dSceneService,
};

pub struct Lighting2dPlugin;

impl RuntimePlugin for Lighting2dPlugin {
    fn name(&self) -> &'static str {
        LIGHTING_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(GlobalLight2dSceneService::default())?;
        registry.register(LightMap2dSceneService::default())?;
        registry.register(LightGroup2dSceneService::default())?;
        register_domain_plugin(
            registry,
            LIGHTING_2D_PLUGIN_LABEL,
            &["rendering_2d"],
            &[LIGHTING_2D_CAPABILITY],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
