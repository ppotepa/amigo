use amigo_input_api::KeyCode;

pub fn string_array(values: Vec<String>) -> rhai::Array {
    values.into_iter().map(Into::into).collect()
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
        KeyCode::R => "R".to_owned(),
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
        "R" => KeyCode::R,
        _ => KeyCode::Unknown,
    }
}
