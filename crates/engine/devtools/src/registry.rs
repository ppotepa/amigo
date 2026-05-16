use std::sync::{Arc, Mutex};

use crate::{ConsoleCommandDescriptor, ConsoleCommandSchema, ParsedConsoleCommand};

pub trait ConsoleCommandSpec: Send + Sync {
    fn name(&self) -> &'static str;

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor>;

    fn schemas(&self) -> Vec<ConsoleCommandSchema> {
        Vec::new()
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool;
}

pub struct ConsoleCommandRegistry<H: ?Sized> {
    handlers: Mutex<Vec<Arc<H>>>,
}

impl<H: ?Sized> Default for ConsoleCommandRegistry<H> {
    fn default() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }
}

impl<H: ?Sized> ConsoleCommandRegistry<H> {
    pub fn register_arc(&self, handler: Arc<H>) {
        self.handlers
            .lock()
            .expect("console command registry mutex should not be poisoned")
            .push(handler);
    }

    pub fn handlers(&self) -> Vec<Arc<H>> {
        self.handlers
            .lock()
            .expect("console command registry mutex should not be poisoned")
            .clone()
    }
}

impl<H: ?Sized + ConsoleCommandSpec> ConsoleCommandRegistry<H> {
    pub fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        self.handlers()
            .into_iter()
            .flat_map(|handler| handler.descriptors())
            .collect()
    }

    pub fn schemas(&self) -> Vec<ConsoleCommandSchema> {
        self.handlers()
            .into_iter()
            .flat_map(|handler| handler.schemas())
            .collect()
    }

    pub fn handler_for(&self, command: &ParsedConsoleCommand) -> Option<Arc<H>> {
        self.handlers()
            .into_iter()
            .find(|handler| handler.can_handle(command))
    }
}
