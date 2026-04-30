use std::sync::Mutex;

use amigo_math::Vec2;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiInputSnapshot {
    pub mouse_position: Option<Vec2>,
    pub mouse_left_down: bool,
    pub mouse_left_pressed: bool,
    pub mouse_left_released: bool,
    pub mouse_wheel_y: f32,
}

#[derive(Debug, Default)]
pub struct UiInputService {
    snapshot: Mutex<UiInputSnapshot>,
    active_path: Mutex<Option<String>>,
}

impl UiInputService {
    pub fn set_mouse_position(&self, x: f32, y: f32) {
        self.snapshot
            .lock()
            .expect("ui input mutex should not be poisoned")
            .mouse_position = Some(Vec2::new(x, y));
    }

    pub fn set_left_button(&self, pressed: bool) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("ui input mutex should not be poisoned");

        if pressed && !snapshot.mouse_left_down {
            snapshot.mouse_left_pressed = true;
        }

        if !pressed && snapshot.mouse_left_down {
            snapshot.mouse_left_released = true;
        }

        snapshot.mouse_left_down = pressed;
    }

    pub fn add_mouse_wheel(&self, delta_y: f32) {
        self.snapshot
            .lock()
            .expect("ui input mutex should not be poisoned")
            .mouse_wheel_y += delta_y;
    }

    pub fn snapshot(&self) -> UiInputSnapshot {
        self.snapshot
            .lock()
            .expect("ui input mutex should not be poisoned")
            .clone()
    }

    pub fn clear_frame_transients(&self) {
        let mut snapshot = self
            .snapshot
            .lock()
            .expect("ui input mutex should not be poisoned");
        snapshot.mouse_left_pressed = false;
        snapshot.mouse_left_released = false;
        snapshot.mouse_wheel_y = 0.0;
    }

    pub fn set_active_path(&self, path: Option<String>) {
        *self
            .active_path
            .lock()
            .expect("ui input active path mutex should not be poisoned") = path;
    }

    pub fn active_path(&self) -> Option<String> {
        self.active_path
            .lock()
            .expect("ui input active path mutex should not be poisoned")
            .clone()
    }
}
