use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use amigo_core::{AmigoError, AmigoResult};

use crate::{PluginSceneCommand, RuntimeSceneCommandHandler, SceneCommand};

#[derive(Default)]
pub struct ScenePluginCommandHandlerRegistry {
    handlers: RwLock<BTreeMap<String, Arc<dyn RuntimeSceneCommandHandler>>>,
}

impl ScenePluginCommandHandlerRegistry {
    pub fn register(
        &self,
        command_type: impl Into<String>,
        handler: Arc<dyn RuntimeSceneCommandHandler>,
    ) {
        self.try_register(command_type, handler)
            .expect("duplicate plugin scene command handler");
    }

    pub fn try_register(
        &self,
        command_type: impl Into<String>,
        handler: Arc<dyn RuntimeSceneCommandHandler>,
    ) -> AmigoResult<()> {
        let command_type = command_type.into();
        let mut handlers = self
            .handlers
            .write()
            .expect("plugin scene command handler registry poisoned");
        if handlers.contains_key(&command_type) {
            return Err(AmigoError::Message(format!(
                "duplicate plugin scene command handler `{command_type}`"
            )));
        }

        handlers.insert(command_type, handler);
        Ok(())
    }

    pub fn handler_for(&self, command_type: &str) -> Option<Arc<dyn RuntimeSceneCommandHandler>> {
        self.handlers
            .read()
            .expect("plugin scene command handler registry poisoned")
            .get(command_type)
            .cloned()
    }

    pub fn dispatch(
        &self,
        runtime: &amigo_runtime::Runtime,
        command: SceneCommand,
    ) -> AmigoResult<Option<()>> {
        let SceneCommand::Plugin {
            command: PluginSceneCommand { command_type, .. },
        } = &command
        else {
            return Ok(None);
        };

        let Some(handler) = self.handler_for(command_type) else {
            return Err(AmigoError::Message(format!(
                "unhandled plugin scene command `{command_type}`"
            )));
        };

        handler.handle(runtime, command)?;
        Ok(Some(()))
    }
}
