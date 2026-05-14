use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{POST_FX_2D_CAPABILITY, POST_FX_2D_PLUGIN_LABEL, PostFx2dService};

pub struct PostFx2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct PostFx2dPlugin;

impl RuntimePlugin for PostFx2dPlugin {
    fn name(&self) -> &'static str {
        POST_FX_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(PostFx2dService::default())?;
        registry.register(PostFx2dDomainInfo {
            crate_name: "amigo-2d-post-fx",
            capability: POST_FX_2D_CAPABILITY,
        })?;
        register_domain_plugin(
            registry,
            POST_FX_2D_PLUGIN_LABEL,
            &[POST_FX_2D_CAPABILITY],
            &["rendering_2d"],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
