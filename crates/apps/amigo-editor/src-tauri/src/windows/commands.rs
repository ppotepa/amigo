use tauri::AppHandle;

use super::descriptors::EditorWindowKind;
use super::manager::open_or_focus_window;

pub fn open_theme_window(app: AppHandle) -> Result<(), String> {
    open_or_focus_window(&app, EditorWindowKind::Theme)?;
    Ok(())
}

pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    open_or_focus_window(&app, EditorWindowKind::Settings)?;
    Ok(())
}

pub fn open_mod_settings_window(app: AppHandle, session_id: String) -> Result<(), String> {
    open_or_focus_window(&app, EditorWindowKind::ModSettings { session_id })?;
    Ok(())
}
