use std::fs;
use std::path::PathBuf;

use crate::dto::EditorSettingsDto;
use crate::settings::theme::{normalize_font_id, normalize_theme_id};

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

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return default;
    };

    let settings = migrate_editor_settings_value(value).unwrap_or(default);
    let _ = save_editor_settings(&settings);
    settings
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
        settings_version: 1,
        mods_root: None,
        cache_root_override: None,
        active_theme_id: "night-in-mexico".to_owned(),
        active_font_id: "source-sans-3".to_owned(),
        last_opened_mod_id: None,
    }
}

fn migrate_editor_settings_value(
    mut value: serde_json::Value,
) -> Result<EditorSettingsDto, String> {
    let Some(object) = value.as_object_mut() else {
        return Ok(default_editor_settings());
    };

    object.insert("settingsVersion".to_owned(), serde_json::Value::from(1));

    let theme = object
        .get("activeThemeId")
        .and_then(|value| value.as_str())
        .and_then(normalize_theme_id)
        .unwrap_or("night-in-mexico");
    object.insert(
        "activeThemeId".to_owned(),
        serde_json::Value::from(theme.to_owned()),
    );

    let font = object
        .get("activeFontId")
        .and_then(|value| value.as_str())
        .and_then(normalize_font_id)
        .unwrap_or("source-sans-3");
    object.insert(
        "activeFontId".to_owned(),
        serde_json::Value::from(font.to_owned()),
    );

    serde_json::from_value(value)
        .map_err(|error| format!("failed to migrate editor settings: {error}"))
}

#[cfg(test)]
mod tests {
    use super::migrate_editor_settings_value;

    #[test]
    fn migrates_invalid_theme_and_missing_font() {
        let value = serde_json::json!({
            "activeThemeId": "unknown-theme",
            "lastOpenedModId": "ink-wars"
        });

        let settings = migrate_editor_settings_value(value).expect("settings should migrate");

        assert_eq!(settings.settings_version, 1);
        assert_eq!(settings.active_theme_id, "night-in-mexico");
        assert_eq!(settings.active_font_id, "source-sans-3");
        assert_eq!(settings.last_opened_mod_id.as_deref(), Some("ink-wars"));
    }
}
