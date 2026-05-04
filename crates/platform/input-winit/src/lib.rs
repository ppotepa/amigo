//! Winit-backed input backend.
//! It translates desktop window events into the engine input API used by gameplay systems and scripts.

use amigo_input_api::{
    InputBackend, InputEvent, InputModifiers, InputServiceInfo, InputState, KeyCode, MouseButton,
};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, Copy, Default)]
pub struct WinitInputBackend;

impl WinitInputBackend {
    pub fn is_pressed(state: winit::event::ElementState) -> bool {
        matches!(state, winit::event::ElementState::Pressed)
    }
}

pub fn map_key_code(key_code: winit::keyboard::KeyCode) -> KeyCode {
    match key_code {
        winit::keyboard::KeyCode::Escape => KeyCode::Escape,
        winit::keyboard::KeyCode::Enter => KeyCode::Enter,
        winit::keyboard::KeyCode::Space => KeyCode::Space,
        winit::keyboard::KeyCode::KeyW => KeyCode::W,
        winit::keyboard::KeyCode::KeyA => KeyCode::A,
        winit::keyboard::KeyCode::KeyB => KeyCode::B,
        winit::keyboard::KeyCode::KeyS => KeyCode::S,
        winit::keyboard::KeyCode::KeyD => KeyCode::D,
        winit::keyboard::KeyCode::KeyR => KeyCode::R,
        winit::keyboard::KeyCode::KeyT => KeyCode::T,
        winit::keyboard::KeyCode::F1 => KeyCode::F1,
        winit::keyboard::KeyCode::F2 => KeyCode::F2,
        winit::keyboard::KeyCode::Digit1 => KeyCode::Digit1,
        winit::keyboard::KeyCode::Digit2 => KeyCode::Digit2,
        winit::keyboard::KeyCode::Digit3 => KeyCode::Digit3,
        winit::keyboard::KeyCode::Digit4 => KeyCode::Digit4,
        winit::keyboard::KeyCode::Digit5 => KeyCode::Digit5,
        winit::keyboard::KeyCode::Digit6 => KeyCode::Digit6,
        winit::keyboard::KeyCode::Digit7 => KeyCode::Digit7,
        winit::keyboard::KeyCode::Digit8 => KeyCode::Digit8,
        winit::keyboard::KeyCode::Digit9 => KeyCode::Digit9,
        winit::keyboard::KeyCode::Digit0 => KeyCode::Digit0,
        winit::keyboard::KeyCode::ArrowUp => KeyCode::Up,
        winit::keyboard::KeyCode::ArrowDown => KeyCode::Down,
        winit::keyboard::KeyCode::ArrowLeft => KeyCode::Left,
        winit::keyboard::KeyCode::ArrowRight => KeyCode::Right,
        _ => KeyCode::Unknown,
    }
}

pub fn map_key_event(event: &winit::event::KeyEvent) -> Option<InputEvent> {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(code) => Some(InputEvent::Key {
            key: map_key_code(code),
            pressed: WinitInputBackend::is_pressed(event.state),
        }),
        _ => None,
    }
}

pub fn map_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Other(4),
        winit::event::MouseButton::Forward => MouseButton::Other(5),
        winit::event::MouseButton::Other(value) => MouseButton::Other(value),
    }
}

pub fn map_modifiers(modifiers: winit::keyboard::ModifiersState) -> InputModifiers {
    InputModifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        super_key: modifiers.super_key(),
    }
}

impl InputBackend for WinitInputBackend {
    fn backend_name(&self) -> &'static str {
        "winit"
    }
}

pub struct WinitInputPlugin;

impl RuntimePlugin for WinitInputPlugin {
    fn name(&self) -> &'static str {
        "amigo-input-winit"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        let _ = WinitInputBackend::is_pressed(winit::event::ElementState::Pressed);

        registry.register(InputServiceInfo {
            backend_name: "winit",
            gamepad_support: false,
        })?;
        registry.register(InputState::default())?;
        registry.register(WinitInputBackend)
    }
}
