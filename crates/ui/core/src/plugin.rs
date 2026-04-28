use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::service::register_ui_services;

pub struct UiPlugin;

impl RuntimePlugin for UiPlugin {
    fn name(&self) -> &'static str {
        "amigo-ui"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_ui_services(registry)
    }
}
