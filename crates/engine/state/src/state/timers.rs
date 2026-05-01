#[derive(Debug, Clone, PartialEq)]
pub struct SceneTimer {
    pub duration_seconds: f32,
    pub elapsed_seconds: f32,
}

impl SceneTimer {
    pub fn active(&self) -> bool {
        self.elapsed_seconds < self.duration_seconds
    }

    pub fn ready(&self) -> bool {
        !self.active()
    }
}

#[derive(Debug, Default)]
pub struct SceneTimerService {
    timers: Mutex<BTreeMap<StateKey, SceneTimer>>,
}

impl SceneTimerService {
    pub fn start(&self, key: impl Into<String>, duration_seconds: f32) -> bool {
        let key = StateKey::scene(key);
        if key.name.is_empty() || !duration_seconds.is_finite() || duration_seconds < 0.0 {
            return false;
        }
        self.timers
            .lock()
            .expect("scene timer service mutex should not be poisoned")
            .insert(
                key,
                SceneTimer {
                    duration_seconds,
                    elapsed_seconds: 0.0,
                },
            );
        true
    }

    pub fn ready(&self, key: &str) -> bool {
        self.timer(key).is_some_and(|timer| timer.ready())
    }

    pub fn active(&self, key: &str) -> bool {
        self.timer(key).is_some_and(|timer| timer.active())
    }

    pub fn after(&self, key: impl Into<String>, duration_seconds: f32) -> bool {
        let key = key.into();
        if key.is_empty() || !duration_seconds.is_finite() || duration_seconds < 0.0 {
            return false;
        }
        if self.ready(&key) {
            self.remove(&key);
            return true;
        }
        if !self.contains(&key) {
            self.start(key, duration_seconds);
        }
        false
    }

    pub fn tick(&self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        let mut timers = self
            .timers
            .lock()
            .expect("scene timer service mutex should not be poisoned");
        for timer in timers.values_mut() {
            timer.elapsed_seconds =
                (timer.elapsed_seconds + delta_seconds).min(timer.duration_seconds);
        }
    }

    pub fn clear_scene(&self) {
        self.timers
            .lock()
            .expect("scene timer service mutex should not be poisoned")
            .retain(|key, _| key.scope != StateScope::Scene);
    }

    pub fn reset_scene(&self) {
        self.clear_scene();
    }

    fn timer(&self, key: &str) -> Option<SceneTimer> {
        self.timers
            .lock()
            .expect("scene timer service mutex should not be poisoned")
            .get(&StateKey::scene(key))
            .cloned()
    }

    fn contains(&self, key: &str) -> bool {
        self.timers
            .lock()
            .expect("scene timer service mutex should not be poisoned")
            .contains_key(&StateKey::scene(key))
    }

    fn remove(&self, key: &str) -> Option<SceneTimer> {
        self.timers
            .lock()
            .expect("scene timer service mutex should not be poisoned")
            .remove(&StateKey::scene(key))
    }
}

