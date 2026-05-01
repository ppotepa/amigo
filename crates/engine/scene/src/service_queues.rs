use std::sync::Mutex;

use crate::*;

#[derive(Debug, Default)]
pub struct SceneCommandQueue {
    commands: Mutex<Vec<SceneCommand>>,
}

impl SceneCommandQueue {
    pub fn submit(&self, command: SceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.push(command);
    }

    pub fn pending(&self) -> Vec<SceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.clone()
    }

    pub fn drain(&self) -> Vec<SceneCommand> {
        let mut commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.drain(..).collect()
    }
}

#[derive(Debug, Default)]
pub struct SceneEventQueue {
    events: Mutex<Vec<SceneEvent>>,
}

impl SceneEventQueue {
    pub fn publish(&self, event: SceneEvent) {
        let mut events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.push(event);
    }

    pub fn pending(&self) -> Vec<SceneEvent> {
        let events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.clone()
    }

    pub fn drain(&self) -> Vec<SceneEvent> {
        let mut events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.drain(..).collect()
    }
}
