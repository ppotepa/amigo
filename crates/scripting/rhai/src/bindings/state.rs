use std::sync::Arc;

use amigo_state::SceneStateService;

#[derive(Clone)]
pub struct StateApi {
    pub(crate) state: Option<Arc<SceneStateService>>,
}

impl StateApi {
    pub fn set_int(&mut self, key: &str, value: rhai::INT) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.set_int(key, value as i64))
    }

    pub fn set_float(&mut self, key: &str, value: rhai::FLOAT) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.set_float(key, value as f64))
    }

    pub fn set_bool(&mut self, key: &str, value: bool) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.set_bool(key, value))
    }

    pub fn set_string(&mut self, key: &str, value: &str) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.set_string(key, value))
    }

    pub fn get_int(&mut self, key: &str) -> rhai::INT {
        self.state
            .as_ref()
            .and_then(|state| state.get_int(key))
            .unwrap_or_default() as rhai::INT
    }

    pub fn get_float(&mut self, key: &str) -> rhai::FLOAT {
        self.state
            .as_ref()
            .and_then(|state| state.get_float(key))
            .unwrap_or_default() as rhai::FLOAT
    }

    pub fn get_bool(&mut self, key: &str) -> bool {
        self.state
            .as_ref()
            .and_then(|state| state.get_bool(key))
            .unwrap_or_default()
    }

    pub fn get_string(&mut self, key: &str) -> String {
        self.state
            .as_ref()
            .and_then(|state| state.get_string(key))
            .unwrap_or_default()
    }

    pub fn add_int(&mut self, key: &str, delta: rhai::INT) -> rhai::INT {
        self.state
            .as_ref()
            .map(|state| state.add_int(key, delta as i64) as rhai::INT)
            .unwrap_or_default()
    }

    pub fn add_float(&mut self, key: &str, delta: rhai::FLOAT) -> rhai::FLOAT {
        self.state
            .as_ref()
            .map(|state| state.add_float(key, delta as f64) as rhai::FLOAT)
            .unwrap_or_default()
    }

    pub fn add_bool(&mut self, key: &str, value: bool) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.add_bool(key, value))
    }

    pub fn add_string(&mut self, key: &str, suffix: &str) -> String {
        self.state
            .as_ref()
            .map(|state| state.add_string(key, suffix))
            .unwrap_or_default()
    }

    pub fn reset_scene(&mut self) {
        if let Some(state) = self.state.as_ref() {
            state.clear_scene();
        }
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<StateApi>("WorldState")
        .register_fn("set_int", StateApi::set_int)
        .register_fn("set_float", StateApi::set_float)
        .register_fn("set_bool", StateApi::set_bool)
        .register_fn("set_string", StateApi::set_string)
        .register_fn("get_int", StateApi::get_int)
        .register_fn("get_float", StateApi::get_float)
        .register_fn("get_bool", StateApi::get_bool)
        .register_fn("get_string", StateApi::get_string)
        .register_fn("add_int", StateApi::add_int)
        .register_fn("add_float", StateApi::add_float)
        .register_fn("add_bool", StateApi::add_bool)
        .register_fn("add_string", StateApi::add_string)
        .register_fn("reset_scene", StateApi::reset_scene);
}
