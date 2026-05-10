use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{
    COMPOSITION_2D_CAPABILITY, COMPOSITION_2D_PLUGIN_LABEL, LightRoute2dSceneService,
    RenderLayer2dSceneService,
};

pub struct Composition2dPlugin;

impl RuntimePlugin for Composition2dPlugin {
    fn name(&self) -> &'static str {
        COMPOSITION_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(RenderLayer2dSceneService::default())?;
        registry.register(LightRoute2dSceneService::default())?;
        register_domain_plugin(
            registry,
            COMPOSITION_2D_PLUGIN_LABEL,
            &["rendering_2d"],
            &[COMPOSITION_2D_CAPABILITY],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
