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
