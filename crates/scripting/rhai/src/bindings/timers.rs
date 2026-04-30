use std::sync::Arc;

use amigo_state::SceneTimerService;

#[derive(Clone)]
pub struct TimersApi {
    pub(crate) timers: Option<Arc<SceneTimerService>>,
}

impl TimersApi {
    pub fn start(&mut self, key: &str, duration_seconds: rhai::FLOAT) -> bool {
        self.timers
            .as_ref()
            .is_some_and(|timers| timers.start(key, duration_seconds as f32))
    }

    pub fn ready(&mut self, key: &str) -> bool {
        self.timers.as_ref().is_some_and(|timers| timers.ready(key))
    }

    pub fn active(&mut self, key: &str) -> bool {
        self.timers
            .as_ref()
            .is_some_and(|timers| timers.active(key))
    }

    pub fn after(&mut self, key: &str, duration_seconds: rhai::FLOAT) -> bool {
        self.timers
            .as_ref()
            .is_some_and(|timers| timers.after(key, duration_seconds as f32))
    }

    pub fn tick(&mut self, delta_seconds: rhai::FLOAT) {
        if let Some(timers) = self.timers.as_ref() {
            timers.tick(delta_seconds as f32);
        }
    }

    pub fn advance(&mut self, delta_seconds: rhai::FLOAT) {
        self.tick(delta_seconds);
    }

    pub fn reset_scene(&mut self) {
        if let Some(timers) = self.timers.as_ref() {
            timers.reset_scene();
        }
    }
}
