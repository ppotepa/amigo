use std::sync::Mutex;

use crate::{DepthAuxMap2dDrawCommand, DepthMap2dDrawCommand};

#[derive(Default)]
pub struct DepthMap2dSceneService {
    commands: Mutex<Vec<DepthMap2dDrawCommand>>,
    aux_commands: Mutex<Vec<DepthAuxMap2dDrawCommand>>,
}

impl DepthMap2dSceneService {
    pub fn queue(&self, command: DepthMap2dDrawCommand) {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .push(command);
    }

    pub fn queue_aux(&self, command: DepthAuxMap2dDrawCommand) {
        self.aux_commands
            .lock()
            .expect("depth aux map registry mutex should not be poisoned")
            .push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .clear();
        self.aux_commands
            .lock()
            .expect("depth aux map registry mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<DepthMap2dDrawCommand> {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .clone()
    }

    pub fn aux_commands(&self) -> Vec<DepthAuxMap2dDrawCommand> {
        self.aux_commands
            .lock()
            .expect("depth aux map registry mutex should not be poisoned")
            .clone()
    }
}
