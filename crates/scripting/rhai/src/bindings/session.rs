use std::sync::Arc;

use amigo_state::SessionStateService;

#[derive(Clone)]
pub struct SessionApi {
    pub(crate) session: Option<Arc<SessionStateService>>,
}

impl SessionApi {
    pub fn set_int(&mut self, key: &str, value: rhai::INT) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.set_int(key, value as i64))
    }

    pub fn set_float(&mut self, key: &str, value: rhai::FLOAT) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.set_float(key, value as f64))
    }

    pub fn set_bool(&mut self, key: &str, value: bool) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.set_bool(key, value))
    }

    pub fn set_string(&mut self, key: &str, value: &str) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.set_string(key, value))
    }

    pub fn get_int(&mut self, key: &str) -> rhai::INT {
        self.session
            .as_ref()
            .and_then(|session| session.get_int(key))
            .unwrap_or_default() as rhai::INT
    }

    pub fn get_float(&mut self, key: &str) -> rhai::FLOAT {
        self.session
            .as_ref()
            .and_then(|session| session.get_float(key))
            .unwrap_or_default() as rhai::FLOAT
    }

    pub fn get_bool(&mut self, key: &str) -> bool {
        self.session
            .as_ref()
            .and_then(|session| session.get_bool(key))
            .unwrap_or_default()
    }

    pub fn get_string(&mut self, key: &str) -> String {
        self.session
            .as_ref()
            .and_then(|session| session.get_string(key))
            .unwrap_or_default()
    }

    pub fn add_int(&mut self, key: &str, delta: rhai::INT) -> rhai::INT {
        self.session
            .as_ref()
            .map(|session| session.add_int(key, delta as i64) as rhai::INT)
            .unwrap_or_default()
    }

    pub fn add_float(&mut self, key: &str, delta: rhai::FLOAT) -> rhai::FLOAT {
        self.session
            .as_ref()
            .map(|session| session.add_float(key, delta as f64) as rhai::FLOAT)
            .unwrap_or_default()
    }

    pub fn add_bool(&mut self, key: &str, value: bool) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.add_bool(key, value))
    }

    pub fn add_string(&mut self, key: &str, suffix: &str) -> String {
        self.session
            .as_ref()
            .map(|session| session.add_string(key, suffix))
            .unwrap_or_default()
    }
}
