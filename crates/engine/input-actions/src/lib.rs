use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_input_api::{InputState, KeyCode};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputActionId(pub String);

impl InputActionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputActionMap {
    pub id: String,
    pub actions: BTreeMap<InputActionId, InputActionBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputActionBinding {
    Axis {
        positive: Vec<KeyCode>,
        negative: Vec<KeyCode>,
    },
    Button {
        pressed: Vec<KeyCode>,
    },
}

#[derive(Debug, Default)]
pub struct InputActionService {
    maps: Mutex<BTreeMap<String, InputActionMap>>,
    active_map: Mutex<Option<String>>,
}

impl InputActionService {
    pub fn register_map(&self, map: InputActionMap, active: bool) {
        let id = map.id.clone();
        self.maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .insert(id.clone(), map);

        if active {
            self.set_active_map(&id);
        }
    }

    pub fn set_active_map(&self, id: &str) -> bool {
        if !self
            .maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .contains_key(id)
        {
            return false;
        }

        *self
            .active_map
            .lock()
            .expect("input action service active map mutex should not be poisoned") =
            Some(id.to_owned());
        true
    }

    pub fn active_map_id(&self) -> Option<String> {
        self.active_map
            .lock()
            .expect("input action service active map mutex should not be poisoned")
            .clone()
    }

    pub fn map(&self, id: &str) -> Option<InputActionMap> {
        self.maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn maps(&self) -> Vec<InputActionMap> {
        self.maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .clear();
        self.active_map
            .lock()
            .expect("input action service active map mutex should not be poisoned")
            .take();
    }

    pub fn axis(&self, input: &InputState, action: &str) -> f32 {
        let Some(binding) = self.active_binding(action) else {
            return 0.0;
        };

        match binding {
            InputActionBinding::Axis { positive, negative } => {
                let positive = positive.iter().any(|key| input.is_down(*key));
                let negative = negative.iter().any(|key| input.is_down(*key));
                match (positive, negative) {
                    (true, false) => 1.0,
                    (false, true) => -1.0,
                    _ => 0.0,
                }
            }
            InputActionBinding::Button { pressed } => {
                if pressed.iter().any(|key| input.is_down(*key)) {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn down(&self, input: &InputState, action: &str) -> bool {
        let Some(binding) = self.active_binding(action) else {
            return false;
        };

        match binding {
            InputActionBinding::Axis { positive, negative } => positive
                .iter()
                .chain(negative.iter())
                .any(|key| input.is_down(*key)),
            InputActionBinding::Button { pressed } => pressed.iter().any(|key| input.is_down(*key)),
        }
    }

    pub fn pressed(&self, input: &InputState, action: &str) -> bool {
        let Some(binding) = self.active_binding(action) else {
            return false;
        };

        match binding {
            InputActionBinding::Axis { positive, negative } => positive
                .iter()
                .chain(negative.iter())
                .any(|key| input.was_pressed(*key)),
            InputActionBinding::Button { pressed } => {
                pressed.iter().any(|key| input.was_pressed(*key))
            }
        }
    }

    fn active_binding(&self, action: &str) -> Option<InputActionBinding> {
        let active_map_id = self.active_map_id()?;
        self.maps
            .lock()
            .expect("input action service map mutex should not be poisoned")
            .get(&active_map_id)
            .and_then(|map| map.actions.get(&InputActionId::new(action)))
            .cloned()
    }
}

pub fn parse_key_code(value: &str) -> KeyCode {
    match value.trim() {
        "ArrowLeft" | "Left" => KeyCode::Left,
        "ArrowRight" | "Right" => KeyCode::Right,
        "ArrowUp" | "Up" => KeyCode::Up,
        "ArrowDown" | "Down" => KeyCode::Down,
        "Escape" | "Esc" => KeyCode::Escape,
        "Enter" | "Return" => KeyCode::Enter,
        "Space" | "Spacebar" => KeyCode::Space,
        "W" | "KeyW" => KeyCode::W,
        "A" | "KeyA" => KeyCode::A,
        "B" | "KeyB" => KeyCode::B,
        "S" | "KeyS" => KeyCode::S,
        "D" | "KeyD" => KeyCode::D,
        "R" | "KeyR" => KeyCode::R,
        "T" | "KeyT" => KeyCode::T,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "1" | "Key1" | "Digit1" => KeyCode::Digit1,
        "2" | "Key2" | "Digit2" => KeyCode::Digit2,
        "3" | "Key3" | "Digit3" => KeyCode::Digit3,
        "4" | "Key4" | "Digit4" => KeyCode::Digit4,
        "5" | "Key5" | "Digit5" => KeyCode::Digit5,
        "6" | "Key6" | "Digit6" => KeyCode::Digit6,
        "7" | "Key7" | "Digit7" => KeyCode::Digit7,
        "8" | "Key8" | "Digit8" => KeyCode::Digit8,
        "9" | "Key9" | "Digit9" => KeyCode::Digit9,
        "0" | "Key0" | "Digit0" => KeyCode::Digit0,
        _ => KeyCode::Unknown,
    }
}

pub struct InputActionPlugin;

impl RuntimePlugin for InputActionPlugin {
    fn name(&self) -> &'static str {
        "amigo-input-actions"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(InputActionService::default())
    }
}

#[cfg(test)]
include!("tests.rs");
