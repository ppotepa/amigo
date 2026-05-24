use crate::state::AppState;
use std::{fs, path::PathBuf};

fn settings_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("amigo-char-3d").join("settings-v7.json")
}

pub fn load() -> AppState {
    let path = settings_path();
    let Ok(text) = fs::read_to_string(path) else {
        return AppState::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(state: &AppState) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, text);
    }
}
