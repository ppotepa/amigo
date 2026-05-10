use std::sync::{Arc, Mutex};

use super::dispatcher::ConsoleCommandContext;
use super::model::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

pub(crate) trait ConsoleCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor>;

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool;

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult;
}

#[derive(Default)]
pub(crate) struct ConsoleCommandRegistry {
    handlers: Mutex<Vec<Arc<dyn ConsoleCommandHandler>>>,
}

impl ConsoleCommandRegistry {
    pub(crate) fn register<H>(&self, handler: H)
    where
        H: ConsoleCommandHandler + 'static,
    {
        let _ = handler.name();
        self.handlers
            .lock()
            .expect("console command registry mutex should not be poisoned")
            .push(Arc::new(handler));
    }

    pub(crate) fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        self.handlers
            .lock()
            .expect("console command registry mutex should not be poisoned")
            .iter()
            .flat_map(|handler| handler.descriptors())
            .collect()
    }

    pub(crate) fn handler_for(
        &self,
        command: &ParsedConsoleCommand,
    ) -> Option<Arc<dyn ConsoleCommandHandler>> {
        self.handlers
            .lock()
            .expect("console command registry mutex should not be poisoned")
            .iter()
            .find(|handler| handler.can_handle(command))
            .cloned()
    }
}
