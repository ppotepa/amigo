use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::AssetCatalog;

pub struct AssetsPlugin;

impl RuntimePlugin for AssetsPlugin {
    fn name(&self) -> &'static str {
        "amigo-assets"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AssetCatalog::default())?;
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::AssetScriptCommandHandler,
        );
        Ok(())
    }
}
