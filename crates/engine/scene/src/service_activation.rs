use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::*;

#[derive(Debug, Default)]
pub struct ActivationSetSceneService {
    sets: Mutex<BTreeMap<String, ActivationSetSceneCommand>>,
}

impl ActivationSetSceneService {
    pub fn queue(&self, command: ActivationSetSceneCommand) {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .insert(command.id.clone(), command);
    }

    pub fn clear(&self) {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .clear();
    }

    pub fn activation_set(&self, id: &str) -> Option<ActivationSetSceneCommand> {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn sets(&self) -> Vec<ActivationSetSceneCommand> {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }
}
