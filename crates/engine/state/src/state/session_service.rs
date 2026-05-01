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
