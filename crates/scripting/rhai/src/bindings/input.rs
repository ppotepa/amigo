use std::sync::Arc;

use amigo_input_api::InputState;

use crate::bindings::common::{key_code_name, parse_key_code, string_array};

#[derive(Clone)]
pub struct InputApi {
    pub(crate) input_state: Option<Arc<InputState>>,
}

impl InputApi {
    pub fn down(&mut self, key: &str) -> bool {
        input_down(self.input_state.as_ref(), key)
    }

    pub fn pressed(&mut self, key: &str) -> bool {
        input_pressed(self.input_state.as_ref(), key)
    }

    pub fn any_down(&mut self, keys: &str) -> bool {
        input_any_down(self.input_state.as_ref(), keys_from_csv(keys))
    }

    pub fn any_down_array(&mut self, keys: rhai::Array) -> bool {
        input_any_down(self.input_state.as_ref(), keys_from_array(keys))
    }

    pub fn any_pressed(&mut self, keys: &str) -> bool {
        input_any_pressed(self.input_state.as_ref(), keys_from_csv(keys))
    }

    pub fn any_pressed_array(&mut self, keys: rhai::Array) -> bool {
        input_any_pressed(self.input_state.as_ref(), keys_from_array(keys))
    }

    pub fn axis(&mut self, positive_keys: &str, negative_keys: &str) -> rhai::INT {
        input_axis(
            self.input_state.as_ref(),
            keys_from_csv(positive_keys),
            keys_from_csv(negative_keys),
        )
    }

    pub fn axis_array(
        &mut self,
        positive_keys: rhai::Array,
        negative_keys: rhai::Array,
    ) -> rhai::INT {
        input_axis(
            self.input_state.as_ref(),
            keys_from_array(positive_keys),
            keys_from_array(negative_keys),
        )
    }

    pub fn keys(&mut self) -> rhai::Array {
        input_keys(self.input_state.as_ref())
    }
}

pub fn input_down(input_state: Option<&Arc<InputState>>, key: &str) -> bool {
    input_state
        .map(|input_state| input_state.is_down(parse_key_code(key)))
        .unwrap_or(false)
}

pub fn input_pressed(input_state: Option<&Arc<InputState>>, key: &str) -> bool {
    input_state
        .map(|input_state| input_state.was_pressed(parse_key_code(key)))
        .unwrap_or(false)
}

pub fn input_any_down(input_state: Option<&Arc<InputState>>, keys: Vec<String>) -> bool {
    keys.iter().any(|key| input_down(input_state, key.as_str()))
}

pub fn input_any_pressed(input_state: Option<&Arc<InputState>>, keys: Vec<String>) -> bool {
    keys.iter()
        .any(|key| input_pressed(input_state, key.as_str()))
}

pub fn input_axis(
    input_state: Option<&Arc<InputState>>,
    positive_keys: Vec<String>,
    negative_keys: Vec<String>,
) -> rhai::INT {
    let positive = input_any_down(input_state, positive_keys);
    let negative = input_any_down(input_state, negative_keys);
    match (positive, negative) {
        (true, false) => 1,
        (false, true) => -1,
        _ => 0,
    }
}

pub fn input_keys(input_state: Option<&Arc<InputState>>) -> rhai::Array {
    string_array(
        input_state
            .map(|input_state| {
                input_state
                    .pressed_keys()
                    .into_iter()
                    .map(key_code_name)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    )
}

fn keys_from_csv(keys: &str) -> Vec<String> {
    keys.split(',')
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
        .collect()
}

fn keys_from_array(keys: rhai::Array) -> Vec<String> {
    keys.into_iter()
        .filter_map(|key| key.try_cast::<String>())
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty())
        .collect()
}
