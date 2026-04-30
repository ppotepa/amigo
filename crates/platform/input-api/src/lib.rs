use std::collections::BTreeSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyCode {
    Unknown,
    Escape,
    Enter,
    Space,
    W,
    A,
    S,
    D,
    R,
    T,
    F1,
    F2,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Key { key: KeyCode, pressed: bool },
    MouseButton { button: MouseButton, pressed: bool },
    CursorMoved { x: f64, y: f64 },
    MouseWheel { delta_y: f32 },
    ModifiersChanged(InputModifiers),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Debug, Clone)]
pub struct InputServiceInfo {
    pub backend_name: &'static str,
    pub gamepad_support: bool,
}

pub trait InputBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug, Default)]
struct InputSnapshot {
    pressed_keys: BTreeSet<KeyCode>,
    just_pressed_keys: BTreeSet<KeyCode>,
}

#[derive(Debug, Default)]
pub struct InputState {
    snapshot: Mutex<InputSnapshot>,
}

impl InputState {
    pub fn set_key(&self, key: KeyCode, pressed: bool) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("input state mutex should not be poisoned");

        if pressed {
            if snapshot.pressed_keys.insert(key) {
                snapshot.just_pressed_keys.insert(key);
            }
        } else {
            snapshot.pressed_keys.remove(&key);
        }
    }

    pub fn is_down(&self, key: KeyCode) -> bool {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .pressed_keys
            .contains(&key)
    }

    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .just_pressed_keys
            .contains(&key)
    }

    pub fn pressed_keys(&self) -> Vec<KeyCode> {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .pressed_keys
            .iter()
            .copied()
            .collect()
    }

    pub fn clear_frame_transients(&self) {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .just_pressed_keys
            .clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{InputState, KeyCode};

    #[test]
    fn input_state_tracks_arrow_keys() {
        let input = InputState::default();

        input.set_key(KeyCode::Left, true);
        assert!(input.is_down(KeyCode::Left));
        assert!(input.was_pressed(KeyCode::Left));

        input.set_key(KeyCode::Left, false);
        assert!(!input.is_down(KeyCode::Left));
    }

    #[test]
    fn input_state_tracks_just_pressed_per_frame() {
        let input = InputState::default();

        input.set_key(KeyCode::Up, true);
        assert!(input.was_pressed(KeyCode::Up));

        input.clear_frame_transients();
        assert!(!input.was_pressed(KeyCode::Up));
        assert!(input.is_down(KeyCode::Up));

        input.set_key(KeyCode::Up, true);
        assert!(!input.was_pressed(KeyCode::Up));

        input.set_key(KeyCode::Up, false);
        input.set_key(KeyCode::Up, true);
        assert!(input.was_pressed(KeyCode::Up));
    }
}
