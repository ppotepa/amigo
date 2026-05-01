use std::sync::Mutex;

use crate::*;

#[derive(Debug, Default)]
pub struct HydratedSceneState {
    snapshot: Mutex<HydratedSceneSnapshot>,
}

impl HydratedSceneState {
    pub fn snapshot(&self) -> HydratedSceneSnapshot {
        self.snapshot
            .lock()
            .expect("hydrated scene state mutex should not be poisoned")
            .clone()
    }

    pub fn replace(&self, snapshot: HydratedSceneSnapshot) -> HydratedSceneSnapshot {
        let mut state = self
            .snapshot
            .lock()
            .expect("hydrated scene state mutex should not be poisoned");
        std::mem::replace(&mut *state, snapshot)
    }

    pub fn clear(&self) -> HydratedSceneSnapshot {
        self.replace(HydratedSceneSnapshot::default())
    }
}
