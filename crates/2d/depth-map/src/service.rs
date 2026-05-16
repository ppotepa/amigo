use std::sync::Mutex;

use crate::DepthMap2dDrawCommand;

#[derive(Default)]
pub struct DepthMap2dSceneService {
    commands: Mutex<Vec<DepthMap2dDrawCommand>>,
}

impl DepthMap2dSceneService {
    pub fn queue(&self, command: DepthMap2dDrawCommand) {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<DepthMap2dDrawCommand> {
        self.commands
            .lock()
            .expect("depth map registry mutex should not be poisoned")
            .clone()
    }
}
