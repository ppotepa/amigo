use amigo_input_api::KeyCode;

pub fn string_array(values: Vec<String>) -> rhai::Array {
    values.into_iter().map(Into::into).collect()
}

pub fn string_to_float(value: &str) -> rhai::FLOAT {
    value.trim().parse::<rhai::FLOAT>().unwrap_or_default()
}

pub fn string_to_int(value: &str) -> rhai::INT {
    value.trim().parse::<rhai::INT>().unwrap_or_default()
}

pub fn string_to_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

pub fn key_code_name(key: KeyCode) -> String {
    match key {
        KeyCode::Unknown => "Unknown".to_owned(),
        KeyCode::Escape => "Escape".to_owned(),
        KeyCode::Enter => "Enter".to_owned(),
        KeyCode::Space => "Space".to_owned(),
        KeyCode::W => "W".to_owned(),
        KeyCode::A => "A".to_owned(),
        KeyCode::S => "S".to_owned(),
        KeyCode::D => "D".to_owned(),
        KeyCode::B => "B".to_owned(),
        KeyCode::R => "R".to_owned(),
        KeyCode::T => "T".to_owned(),
        KeyCode::F1 => "F1".to_owned(),
        KeyCode::F2 => "F2".to_owned(),
        KeyCode::Digit1 => "Digit1".to_owned(),
        KeyCode::Digit2 => "Digit2".to_owned(),
        KeyCode::Digit3 => "Digit3".to_owned(),
        KeyCode::Digit4 => "Digit4".to_owned(),
        KeyCode::Digit5 => "Digit5".to_owned(),
        KeyCode::Digit6 => "Digit6".to_owned(),
        KeyCode::Digit7 => "Digit7".to_owned(),
        KeyCode::Digit8 => "Digit8".to_owned(),
        KeyCode::Digit9 => "Digit9".to_owned(),
        KeyCode::Digit0 => "Digit0".to_owned(),
        KeyCode::Up => "Up".to_owned(),
        KeyCode::Down => "Down".to_owned(),
        KeyCode::Left => "Left".to_owned(),
        KeyCode::Right => "Right".to_owned(),
    }
}

pub fn parse_key_code(value: &str) -> KeyCode {
    match value {
        "ArrowLeft" | "Left" => KeyCode::Left,
        "ArrowRight" | "Right" => KeyCode::Right,
        "ArrowUp" | "Up" => KeyCode::Up,
        "ArrowDown" | "Down" => KeyCode::Down,
        "Escape" => KeyCode::Escape,
        "Enter" => KeyCode::Enter,
        "Space" => KeyCode::Space,
        "W" => KeyCode::W,
        "A" => KeyCode::A,
        "S" => KeyCode::S,
        "D" => KeyCode::D,
        "B" | "KeyB" => KeyCode::B,
        "R" => KeyCode::R,
        "T" => KeyCode::T,
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
