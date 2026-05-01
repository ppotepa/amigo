use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateScope {
    Scene,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateKey {
    scope: StateScope,
    name: String,
}

impl StateKey {
    pub fn scene(name: impl Into<String>) -> Self {
        Self {
            scope: StateScope::Scene,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneStateValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Default)]
pub struct SceneStateService {
    values: Mutex<BTreeMap<StateKey, SceneStateValue>>,
}

#[derive(Debug, Default)]
pub struct SessionStateService {
    values: Mutex<BTreeMap<String, SceneStateValue>>,
}

impl SceneStateService {
    pub fn set_int(&self, key: impl Into<String>, value: i64) -> bool {
        self.set(StateKey::scene(key), SceneStateValue::Int(value))
    }

    pub fn set_float(&self, key: impl Into<String>, value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        self.set(StateKey::scene(key), SceneStateValue::Float(value))
    }

    pub fn set_bool(&self, key: impl Into<String>, value: bool) -> bool {
        self.set(StateKey::scene(key), SceneStateValue::Bool(value))
    }

    pub fn set_string(&self, key: impl Into<String>, value: impl Into<String>) -> bool {
        self.set(StateKey::scene(key), SceneStateValue::String(value.into()))
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get_scene(key) {
            Some(SceneStateValue::Int(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get_scene(key) {
            Some(SceneStateValue::Float(value)) => Some(value),
            Some(SceneStateValue::Int(value)) => Some(value as f64),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get_scene(key) {
            Some(SceneStateValue::Bool(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get_scene(key) {
            Some(SceneStateValue::String(value)) => Some(value),
            _ => None,
        }
    }

    pub fn add_int(&self, key: impl Into<String>, delta: i64) -> i64 {
        let key = StateKey::scene(key);
        let mut values = self
            .values
            .lock()
            .expect("scene state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Int(value)) => value.saturating_add(delta),
            _ => delta,
        };
        values.insert(key, SceneStateValue::Int(next));
        next
    }

    pub fn add_float(&self, key: impl Into<String>, delta: f64) -> f64 {
        let key_name = key.into();
        if !delta.is_finite() {
            return self.get_float(&key_name).unwrap_or_default();
        }
        let key = StateKey::scene(key_name);
        let mut values = self
            .values
            .lock()
            .expect("scene state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Float(value)) => value + delta,
            Some(SceneStateValue::Int(value)) => *value as f64 + delta,
            _ => delta,
        };
        values.insert(key, SceneStateValue::Float(next));
        next
    }

    pub fn add_bool(&self, key: impl Into<String>, value: bool) -> bool {
        let key = StateKey::scene(key);
        let mut values = self
            .values
            .lock()
            .expect("scene state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Bool(existing)) => *existing || value,
            _ => value,
        };
        values.insert(key, SceneStateValue::Bool(next));
        next
    }

    pub fn add_string(&self, key: impl Into<String>, suffix: impl Into<String>) -> String {
        let key = StateKey::scene(key);
        let suffix = suffix.into();
        let mut values = self
            .values
            .lock()
            .expect("scene state service mutex should not be poisoned");
        let mut next = match values.get(&key) {
            Some(SceneStateValue::String(value)) => value.clone(),
            _ => String::new(),
        };
        next.push_str(&suffix);
        values.insert(key, SceneStateValue::String(next.clone()));
        next
    }

    pub fn clear_scene(&self) {
        self.values
            .lock()
            .expect("scene state service mutex should not be poisoned")
            .retain(|key, _| key.scope != StateScope::Scene);
    }

    fn set(&self, key: StateKey, value: SceneStateValue) -> bool {
        if key.name.is_empty() {
            return false;
        }
        self.values
            .lock()
            .expect("scene state service mutex should not be poisoned")
            .insert(key, value);
        true
    }

    fn get_scene(&self, key: &str) -> Option<SceneStateValue> {
        self.values
            .lock()
            .expect("scene state service mutex should not be poisoned")
            .get(&StateKey::scene(key))
            .cloned()
    }
}

impl SessionStateService {
    pub fn set_int(&self, key: impl Into<String>, value: i64) -> bool {
        self.set(key, SceneStateValue::Int(value))
    }

    pub fn set_float(&self, key: impl Into<String>, value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        self.set(key, SceneStateValue::Float(value))
    }

    pub fn set_bool(&self, key: impl Into<String>, value: bool) -> bool {
        self.set(key, SceneStateValue::Bool(value))
    }

    pub fn set_string(&self, key: impl Into<String>, value: impl Into<String>) -> bool {
        self.set(key, SceneStateValue::String(value.into()))
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(SceneStateValue::Int(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(SceneStateValue::Float(value)) => Some(value),
            Some(SceneStateValue::Int(value)) => Some(value as f64),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(SceneStateValue::Bool(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key) {
            Some(SceneStateValue::String(value)) => Some(value),
            _ => None,
        }
    }

    pub fn add_int(&self, key: impl Into<String>, delta: i64) -> i64 {
        let key = key.into();
        let mut values = self
            .values
            .lock()
            .expect("session state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Int(value)) => value.saturating_add(delta),
            _ => delta,
        };
        values.insert(key, SceneStateValue::Int(next));
        next
    }

    pub fn add_float(&self, key: impl Into<String>, delta: f64) -> f64 {
        let key = key.into();
        if !delta.is_finite() {
            return self.get_float(&key).unwrap_or_default();
        }
        let mut values = self
            .values
            .lock()
            .expect("session state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Float(value)) => value + delta,
            Some(SceneStateValue::Int(value)) => *value as f64 + delta,
            _ => delta,
        };
        values.insert(key, SceneStateValue::Float(next));
        next
    }

    pub fn add_bool(&self, key: impl Into<String>, value: bool) -> bool {
        let key = key.into();
        let mut values = self
            .values
            .lock()
            .expect("session state service mutex should not be poisoned");
        let next = match values.get(&key) {
            Some(SceneStateValue::Bool(existing)) => *existing || value,
            _ => value,
        };
        values.insert(key, SceneStateValue::Bool(next));
        next
    }

    pub fn add_string(&self, key: impl Into<String>, suffix: impl Into<String>) -> String {
        let key = key.into();
        let suffix = suffix.into();
        let mut values = self
            .values
            .lock()
            .expect("session state service mutex should not be poisoned");
        let mut next = match values.get(&key) {
            Some(SceneStateValue::String(value)) => value.clone(),
            _ => String::new(),
        };
        next.push_str(&suffix);
        values.insert(key, SceneStateValue::String(next.clone()));
        next
    }

    pub fn clear(&self) {
        self.values
            .lock()
            .expect("session state service mutex should not be poisoned")
            .clear();
    }

    fn set(&self, key: impl Into<String>, value: SceneStateValue) -> bool {
        let key = key.into();
        if key.is_empty() {
            return false;
        }
        self.values
            .lock()
            .expect("session state service mutex should not be poisoned")
            .insert(key, value);
        true
    }

    fn get(&self, key: &str) -> Option<SceneStateValue> {
        self.values
            .lock()
            .expect("session state service mutex should not be poisoned")
            .get(key)
            .cloned()
    }
}

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

pub struct StatePlugin;

impl RuntimePlugin for StatePlugin {
    fn name(&self) -> &'static str {
        "amigo-state"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        if !registry.has::<SceneStateService>() {
            registry.register(SceneStateService::default())?;
        }
        if !registry.has::<SceneTimerService>() {
            registry.register(SceneTimerService::default())?;
        }
        if !registry.has::<SessionStateService>() {
            registry.register(SessionStateService::default())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SceneStateService, SceneTimerService, SessionStateService};

    #[test]
    fn scene_state_set_get_add_and_clear() {
        let state = SceneStateService::default();

        assert!(state.set_int("score", 10));
        assert!(state.set_float("speed", 1.5));
        assert!(state.set_bool("armed", false));
        assert!(state.set_string("label", "wave"));

        assert_eq!(state.get_int("score"), Some(10));
        assert_eq!(state.add_int("score", 5), 15);
        assert_eq!(state.add_float("speed", 0.25), 1.75);
        assert!(state.add_bool("armed", true));
        assert_eq!(state.add_string("label", " 1"), "wave 1");

        state.clear_scene();

        assert_eq!(state.get_int("score"), None);
        assert_eq!(state.get_string("label"), None);
    }

    #[test]
    fn scene_timers_start_tick_ready_after_and_reset() {
        let timers = SceneTimerService::default();

        assert!(timers.start("cooldown", 0.5));
        assert!(timers.active("cooldown"));
        assert!(!timers.ready("cooldown"));

        timers.tick(0.25);
        assert!(timers.active("cooldown"));

        timers.tick(0.25);
        assert!(!timers.active("cooldown"));
        assert!(timers.ready("cooldown"));

        assert!(!timers.after("spawn", 0.25));
        timers.tick(0.25);
        assert!(timers.after("spawn", 0.25));
        assert!(!timers.active("spawn"));

        timers.reset_scene();
        assert!(!timers.ready("cooldown"));
    }

    #[test]
    fn session_state_survives_scene_state_clear() {
        let scene = SceneStateService::default();
        let session = SessionStateService::default();

        assert!(scene.set_int("score", 120));
        assert!(session.set_bool("game.option", true));
        assert!(session.set_int("game.highscore.1", 10_000));

        scene.clear_scene();

        assert_eq!(scene.get_int("score"), None);
        assert_eq!(session.get_bool("game.option"), Some(true));
        assert_eq!(session.add_int("game.highscore.1", 250), 10_250);
    }
}
