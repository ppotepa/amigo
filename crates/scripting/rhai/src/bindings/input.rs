use std::sync::Arc;

use amigo_input_api::{InputState, MouseButton};

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

    pub fn mouse_down(&mut self, button: &str) -> bool {
        input_mouse_down(self.input_state.as_ref(), button)
    }

    pub fn mouse_pressed(&mut self, button: &str) -> bool {
        input_mouse_pressed(self.input_state.as_ref(), button)
    }

    pub fn mouse_position(&mut self) -> rhai::Map {
        input_mouse_position(self.input_state.as_ref())
    }

    pub fn mouse_canvas_position(
        &mut self,
        canvas_width: rhai::FLOAT,
        canvas_height: rhai::FLOAT,
    ) -> rhai::Map {
        input_mouse_canvas_position(self.input_state.as_ref(), canvas_width, canvas_height)
    }

    pub fn wheel_delta(&mut self) -> rhai::FLOAT {
        input_wheel_delta(self.input_state.as_ref())
    }

    pub fn ctrl_down(&mut self) -> bool {
        input_ctrl_down(self.input_state.as_ref())
    }

    pub fn shift_down(&mut self) -> bool {
        input_shift_down(self.input_state.as_ref())
    }

    pub fn alt_down(&mut self) -> bool {
        input_alt_down(self.input_state.as_ref())
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

pub fn input_mouse_down(input_state: Option<&Arc<InputState>>, button: &str) -> bool {
    input_state
        .map(|input_state| input_state.is_mouse_down(parse_mouse_button(button)))
        .unwrap_or(false)
}

pub fn input_mouse_pressed(input_state: Option<&Arc<InputState>>, button: &str) -> bool {
    input_state
        .map(|input_state| input_state.was_mouse_pressed(parse_mouse_button(button)))
        .unwrap_or(false)
}

pub fn input_mouse_position(input_state: Option<&Arc<InputState>>) -> rhai::Map {
    let (x, y) = input_state
        .and_then(|input_state| input_state.cursor_position())
        .unwrap_or((0.0, 0.0));
    point_map(x, y, input_state.is_some_and(|input_state| input_state.cursor_position().is_some()))
}

pub fn input_mouse_canvas_position(
    input_state: Option<&Arc<InputState>>,
    canvas_width: rhai::FLOAT,
    canvas_height: rhai::FLOAT,
) -> rhai::Map {
    let Some(input_state) = input_state else {
        return point_map(0.0, 0.0, false);
    };
    let Some((x, y)) = input_state.cursor_position() else {
        return point_map(0.0, 0.0, false);
    };
    let (viewport_width, viewport_height) = input_state
        .viewport_size()
        .unwrap_or((canvas_width as f32, canvas_height as f32));
    let canvas_width = canvas_width as f32;
    let canvas_height = canvas_height as f32;
    if canvas_width <= 0.0 || canvas_height <= 0.0 || viewport_width <= 0.0 || viewport_height <= 0.0
    {
        return point_map(0.0, 0.0, false);
    }

    let scale = (viewport_width / canvas_width).max(viewport_height / canvas_height);
    point_map(
        (x - viewport_width * 0.5) / scale,
        (viewport_height * 0.5 - y) / scale,
        true,
    )
}

pub fn input_wheel_delta(input_state: Option<&Arc<InputState>>) -> rhai::FLOAT {
    input_state
        .map(|input_state| input_state.mouse_wheel_delta_y() as rhai::FLOAT)
        .unwrap_or_default()
}

pub fn input_ctrl_down(input_state: Option<&Arc<InputState>>) -> bool {
    input_state
        .map(|input_state| {
            let modifiers = input_state.modifiers();
            modifiers.control || modifiers.super_key
        })
        .unwrap_or(false)
}

pub fn input_shift_down(input_state: Option<&Arc<InputState>>) -> bool {
    input_state
        .map(|input_state| input_state.modifiers().shift)
        .unwrap_or(false)
}

pub fn input_alt_down(input_state: Option<&Arc<InputState>>) -> bool {
    input_state
        .map(|input_state| input_state.modifiers().alt)
        .unwrap_or(false)
}

fn parse_mouse_button(button: &str) -> MouseButton {
    match button.trim().to_ascii_lowercase().as_str() {
        "left" | "mouseleft" | "mouse1" | "primary" => MouseButton::Left,
        "right" | "mouseright" | "mouse2" | "secondary" => MouseButton::Right,
        "middle" | "mousemiddle" | "mouse3" => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

fn point_map(x: f32, y: f32, valid: bool) -> rhai::Map {
    let mut map = rhai::Map::new();
    map.insert("x".into(), (x as rhai::FLOAT).into());
    map.insert("y".into(), (y as rhai::FLOAT).into());
    map.insert("valid".into(), valid.into());
    map
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
