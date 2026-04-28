use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ScriptTimeSnapshot {
    pub delta_seconds: f32,
    pub elapsed_seconds: f32,
    pub frame: u64,
}

#[derive(Debug, Default)]
pub struct ScriptTimeState {
    snapshot: Mutex<ScriptTimeSnapshot>,
}

impl ScriptTimeState {
    pub fn snapshot(&self) -> ScriptTimeSnapshot {
        self.snapshot
            .lock()
            .expect("script time mutex should not be poisoned")
            .clone()
    }

    pub fn advance_frame(&self, delta_seconds: f32) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("script time mutex should not be poisoned");
        snapshot.delta_seconds = delta_seconds.max(0.0);
        snapshot.elapsed_seconds += delta_seconds.max(0.0);
        snapshot.frame += 1;
    }

    pub fn set_passive_delta(&self, delta_seconds: f32) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("script time mutex should not be poisoned");
        snapshot.delta_seconds = delta_seconds.max(0.0);
    }
}

#[derive(Clone)]
pub struct TimeApi {
    pub(crate) state: Arc<ScriptTimeState>,
}

impl TimeApi {
    pub fn delta(&mut self) -> rhai::FLOAT {
        self.state.snapshot().delta_seconds as rhai::FLOAT
    }

    pub fn elapsed(&mut self) -> rhai::FLOAT {
        self.state.snapshot().elapsed_seconds as rhai::FLOAT
    }

    pub fn seconds(&mut self) -> rhai::INT {
        self.state.snapshot().elapsed_seconds.floor() as rhai::INT
    }

    pub fn frame(&mut self) -> rhai::INT {
        self.state.snapshot().frame as rhai::INT
    }
}
