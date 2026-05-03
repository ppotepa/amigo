use std::fs;
use std::path::PathBuf;

use crate::dto::EditorSettingsDto;

const EDITOR_SETTINGS_FILE: &str = "editor-settings.json";
const EDITOR_SETTINGS_DIR: &str = "target/amigo-editor";

pub fn settings_path() -> PathBuf {
    PathBuf::from(EDITOR_SETTINGS_DIR).join(EDITOR_SETTINGS_FILE)
}

pub fn load_editor_settings() -> EditorSettingsDto {
    let default = default_editor_settings();
    let path = settings_path();

    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return default,
    };

    serde_json::from_str(&text).unwrap_or(default)
}

pub fn save_editor_settings(settings: &EditorSettingsDto) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create settings directory `{}`: {error}",
                parent.display()
            )
        })?;
    }

    let text = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("failed to serialize editor settings: {error}"))?;
    fs::write(path, text).map_err(|error| format!("failed to write editor settings: {error}"))
}

pub fn effective_mods_root() -> Option<String> {
    let settings = load_editor_settings();
    settings.mods_root
}

pub fn default_editor_settings() -> EditorSettingsDto {
    EditorSettingsDto {
        mods_root: None,
        cache_root_override: None,
        active_theme_id: "mexico-at-night".to_owned(),
        last_opened_mod_id: None,
    }
}
