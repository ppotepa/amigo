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

