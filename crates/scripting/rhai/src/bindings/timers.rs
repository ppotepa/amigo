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

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<TimersApi>("WorldTimers")
        .register_fn("start", TimersApi::start)
        .register_fn("ready", TimersApi::ready)
        .register_fn("active", TimersApi::active)
        .register_fn("after", TimersApi::after)
        .register_fn("tick", TimersApi::tick)
        .register_fn("advance", TimersApi::advance)
        .register_fn("reset_scene", TimersApi::reset_scene);
}
