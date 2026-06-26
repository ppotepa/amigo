//! Platform-neutral input events and state snapshots.
//! It is the boundary between host backends and gameplay-facing input services.

use std::collections::BTreeSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyCode {
    Unknown,
    Escape,
    Enter,
    Space,
    Backspace,
    Tab,
    Backquote,
    BracketLeft,
    BracketRight,
    Semicolon,
    Quote,
    Period,
    Slash,
    W,
    A,
    S,
    D,
    E,
    F,
    Q,
    B,
    C,
    R,
    T,
    V,
    X,
    Delete,
    Home,
    End,
    F1,
    F2,
    F3,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Key { key: KeyCode, pressed: bool },
    TextInput { text: String },
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
    pressed_mouse_buttons: BTreeSet<MouseButton>,
    just_pressed_mouse_buttons: BTreeSet<MouseButton>,
    cursor_position: Option<(f32, f32)>,
    viewport_size: Option<(f32, f32)>,
    mouse_wheel_delta_y: f32,
    modifiers: InputModifiers,
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

    pub fn set_mouse_button(&self, button: MouseButton, pressed: bool) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("input state mutex should not be poisoned");

        if pressed {
            if snapshot.pressed_mouse_buttons.insert(button) {
                snapshot.just_pressed_mouse_buttons.insert(button);
            }
        } else {
            snapshot.pressed_mouse_buttons.remove(&button);
        }
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .pressed_mouse_buttons
            .contains(&button)
    }

    pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .just_pressed_mouse_buttons
            .contains(&button)
    }

    pub fn set_cursor_position(&self, x: f32, y: f32) {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .cursor_position = Some((x, y));
    }

    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .cursor_position
    }

    pub fn set_viewport_size(&self, width: f32, height: f32) {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .viewport_size = Some((width, height));
    }

    pub fn viewport_size(&self) -> Option<(f32, f32)> {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .viewport_size
    }

    pub fn add_mouse_wheel_delta(&self, delta_y: f32) {
        if !delta_y.is_finite() {
            return;
        }
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .mouse_wheel_delta_y += delta_y;
    }

    pub fn mouse_wheel_delta_y(&self) -> f32 {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .mouse_wheel_delta_y
    }

    pub fn set_modifiers(&self, modifiers: InputModifiers) {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .modifiers = modifiers;
    }

    pub fn modifiers(&self) -> InputModifiers {
        self.snapshot
            .lock()
            .expect("input state mutex should not be poisoned")
            .modifiers
    }

    pub fn clear_frame_transients(&self) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("input state mutex should not be poisoned");
        snapshot.just_pressed_keys.clear();
        snapshot.just_pressed_mouse_buttons.clear();
        snapshot.mouse_wheel_delta_y = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::{InputModifiers, InputState, KeyCode, MouseButton};

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

    #[test]
    fn input_state_tracks_mouse_buttons_and_cursor() {
        let input = InputState::default();

        input.set_cursor_position(120.0, 240.0);
        input.set_viewport_size(1280.0, 720.0);
        input.set_mouse_button(MouseButton::Left, true);

        assert_eq!(input.cursor_position(), Some((120.0, 240.0)));
        assert_eq!(input.viewport_size(), Some((1280.0, 720.0)));
        assert!(input.is_mouse_down(MouseButton::Left));
        assert!(input.was_mouse_pressed(MouseButton::Left));

        input.clear_frame_transients();
        assert!(input.is_mouse_down(MouseButton::Left));
        assert!(!input.was_mouse_pressed(MouseButton::Left));
    }

    #[test]
    fn input_state_tracks_wheel_and_modifiers_as_frame_transients() {
        let input = InputState::default();

        input.add_mouse_wheel_delta(2.0);
        input.add_mouse_wheel_delta(-0.5);
        input.set_modifiers(InputModifiers {
            control: true,
            ..InputModifiers::default()
        });

        assert_eq!(input.mouse_wheel_delta_y(), 1.5);
        assert!(input.modifiers().control);

        input.clear_frame_transients();
        assert_eq!(input.mouse_wheel_delta_y(), 0.0);
        assert!(input.modifiers().control);
    }
}
