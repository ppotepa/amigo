use rfd::FileDialog;
use tauri::AppHandle;

use crate::dto::EditorSettingsDto;
use crate::events::bus;
use crate::settings::editor_settings::{load_editor_settings, save_editor_settings};
use crate::settings::theme::{
    ThemeSettingsDto, normalize_font_id, normalize_theme_id, validate_font_id, validate_theme_id,
};

pub fn get_theme_settings() -> Result<ThemeSettingsDto, String> {
    let settings = load_editor_settings();
    Ok(ThemeSettingsDto {
        active_theme_id: normalize_theme_id(&settings.active_theme_id)
            .unwrap_or("night-in-mexico")
            .to_owned(),
        active_font_id: normalize_font_id(&settings.active_font_id)
            .unwrap_or("source-sans-3")
            .to_owned(),
    })
}

pub fn set_theme_settings(app: AppHandle, theme_id: String) -> Result<ThemeSettingsDto, String> {
    validate_theme_id(&theme_id)?;
    let mut settings = load_editor_settings();
    settings.active_theme_id = theme_id.clone();
    save_editor_settings(&settings)
        .map_err(|error| format!("failed to persist theme settings: {error}"))?;
    let dto = ThemeSettingsDto {
        active_theme_id: settings.active_theme_id,
        active_font_id: settings.active_font_id,
    };
    bus::emit_theme_settings_changed(&app, dto.active_theme_id.clone())?;
    Ok(dto)
}

pub fn set_font_settings(app: AppHandle, font_id: String) -> Result<ThemeSettingsDto, String> {
    validate_font_id(&font_id)?;
    let mut settings = load_editor_settings();
    settings.active_font_id = font_id;
    save_editor_settings(&settings)
        .map_err(|error| format!("failed to persist font settings: {error}"))?;
    let dto = ThemeSettingsDto {
        active_theme_id: settings.active_theme_id,
        active_font_id: settings.active_font_id,
    };
    bus::emit_font_settings_changed(&app, dto.active_font_id.clone())?;
    Ok(dto)
}

pub fn get_editor_settings() -> Result<EditorSettingsDto, String> {
    Ok(load_editor_settings())
}

pub fn set_editor_mods_root(mods_root: String) -> Result<EditorSettingsDto, String> {
    let mut settings = load_editor_settings();
    settings.mods_root = Some(mods_root);
    save_editor_settings(&settings)?;
    Ok(settings)
}

pub fn reset_editor_mods_root() -> Result<EditorSettingsDto, String> {
    let mut settings = load_editor_settings();
    settings.mods_root = None;
    save_editor_settings(&settings)?;
    Ok(settings)
}

pub fn pick_mods_root() -> Result<Option<String>, String> {
    let folder = FileDialog::new()
        .set_title("Choose Mods Root")
        .pick_folder();
    Ok(folder.map(|path| path.display().to_string()))
}
