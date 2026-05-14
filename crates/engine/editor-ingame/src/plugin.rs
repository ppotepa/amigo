use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::commands::IngameEditorConsoleCommandHandler;
use crate::state::IngameEditorState;

pub struct IngameEditorPlugin {
    enabled: bool,
}

impl IngameEditorPlugin {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl RuntimePlugin for IngameEditorPlugin {
    fn name(&self) -> &'static str {
        "amigo-editor-ingame"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(IngameEditorState::new(self.enabled))?;

        if let Some(console_registry) =
            registry.resolve::<amigo_devtools::RuntimeConsoleCommandRegistry>()
        {
            amigo_devtools::register_runtime_console_command_handler(
                console_registry.as_ref(),
                IngameEditorConsoleCommandHandler,
            );
        }

        Ok(())
    }
}
