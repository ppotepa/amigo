use tauri::{AppHandle, Manager, State};

use crate::asset_registry::dto::{
    AssetRegistryDto, CreateAssetDescriptorRequestDto, CreateSpritesheetRulesetRequestDto,
    ManagedAssetDto,
};
use crate::cache::root::EditorPaths;
use crate::dto::{
    CacheInfoDto, CacheMaintenanceResultDto, CachePolicyDto, EditorModDetailsDto,
    EditorModSummaryDto, EditorProjectFileContentDto, EditorProjectStructureTreeDto,
    EditorProjectTreeDto, EditorSceneHierarchyDto, EditorSessionDto, EditorSettingsDto,
    EditorWindowRegistryDto, OpenModResultDto, ScenePreviewDto, WriteProjectFileRequestDto,
};
use crate::session::EditorSessionRegistry;
use crate::sheet::dto::{SheetResourceDto, TileRulesetResourceDto, TilemapResourceDto};
use crate::windows::commands::{
    open_mod_settings_window as open_mod_settings_window_impl,
    open_settings_window as open_settings_window_impl, open_theme_window as open_theme_window_impl,
};
use crate::windows::registry::EditorWindowRegistry;

pub mod assets;
pub mod cache;
pub mod mods;
pub mod preview;
pub mod project_files;
pub mod project_tree;
pub mod session;
pub mod settings;
pub mod shared;
pub mod sheets;
pub mod windows;

#[tauri::command]
pub fn list_known_mods(paths: State<'_, EditorPaths>) -> Result<Vec<EditorModSummaryDto>, String> {
    mods::list_known_mods(paths)
}

#[tauri::command]
pub fn get_mod_details(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    mods::get_mod_details(mod_id, paths)
}

#[tauri::command]
pub async fn request_scene_preview(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    mod_id: String,
    scene_id: String,
    force_regenerate: Option<bool>,
) -> Result<ScenePreviewDto, String> {
    preview::request_scene_preview(app, paths, mod_id, scene_id, force_regenerate).await
}

#[tauri::command]
pub fn open_mod(
    mod_id: String,
    selected_scene_id: Option<String>,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<OpenModResultDto, String> {
    session::open_mod(mod_id, selected_scene_id, sessions)
}

#[tauri::command]
pub fn open_mod_workspace(
    app: AppHandle,
    mod_id: String,
    selected_scene_id: Option<String>,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<OpenModResultDto, String> {
    session::open_mod_workspace(app, mod_id, selected_scene_id, sessions)
}

#[tauri::command]
pub fn open_theme_window(app: AppHandle) -> Result<(), String> {
    open_theme_window_impl(app)
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    open_settings_window_impl(app)
}

#[tauri::command]
pub fn open_mod_settings_window(app: AppHandle, session_id: String) -> Result<(), String> {
    open_mod_settings_window_impl(app, session_id)
}

#[tauri::command]
pub fn register_editor_window(
    registry: State<'_, EditorWindowRegistry>,
    label: String,
    kind: String,
    session_id: Option<String>,
) -> Result<(), String> {
    registry.register_window(label, kind, session_id)
}

#[tauri::command]
pub fn mark_editor_window_focused(
    registry: State<'_, EditorWindowRegistry>,
    label: String,
) -> Result<(), String> {
    registry.mark_focused(&label)
}

#[tauri::command]
pub fn unregister_editor_window(
    registry: State<'_, EditorWindowRegistry>,
    label: String,
) -> Result<(), String> {
    registry.remove_window(&label)
}

#[tauri::command]
pub fn get_window_registry(
    registry: State<'_, EditorWindowRegistry>,
) -> Result<EditorWindowRegistryDto, String> {
    registry.snapshot()
}

#[tauri::command]
pub fn focus_workspace_window(app: AppHandle, session_id: String) -> Result<(), String> {
    let label = format!("workspace-{session_id}");
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("workspace window `{label}` was not found"))?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn close_workspace_window(app: AppHandle, session_id: String) -> Result<(), String> {
    let label = format!("workspace-{session_id}");
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_editor_session(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorSessionDto, String> {
    session::get_editor_session(session_id, sessions)
}

#[tauri::command]
pub fn close_editor_session(
    app: AppHandle,
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<(), String> {
    session::close_editor_session(app, session_id, sessions)
}

#[tauri::command]
pub fn validate_mod(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    mods::validate_mod(mod_id, paths)
}

#[tauri::command]
pub async fn regenerate_all_scene_previews(
    paths: State<'_, EditorPaths>,
    mod_id: String,
) -> Result<Vec<ScenePreviewDto>, String> {
    session::regenerate_all_scene_previews(paths, mod_id).await
}

#[tauri::command]
pub fn reveal_mod_folder(mod_id: String) -> Result<String, String> {
    mods::reveal_mod_folder(mod_id)
}

#[tauri::command]
pub fn reveal_scene_document(mod_id: String, scene_id: String) -> Result<String, String> {
    project_tree::reveal_scene_document(mod_id, scene_id)
}

#[tauri::command]
pub fn get_scene_hierarchy(
    mod_id: String,
    scene_id: String,
) -> Result<EditorSceneHierarchyDto, String> {
    project_tree::get_scene_hierarchy(mod_id, scene_id)
}

#[tauri::command]
pub fn get_project_tree(mod_id: String) -> Result<EditorProjectTreeDto, String> {
    project_tree::get_project_tree(mod_id)
}

#[tauri::command]
pub fn get_project_structure_tree(mod_id: String) -> Result<EditorProjectStructureTreeDto, String> {
    project_tree::get_project_structure_tree(mod_id)
}

#[tauri::command]
pub fn read_project_file(
    mod_id: String,
    relative_path: String,
) -> Result<EditorProjectFileContentDto, String> {
    project_files::read_project_file(mod_id, relative_path)
}

#[tauri::command]
pub fn write_project_file(
    app: AppHandle,
    mod_id: String,
    request: WriteProjectFileRequestDto,
) -> Result<EditorProjectFileContentDto, String> {
    project_files::write_project_file(app, mod_id, request)
}

#[tauri::command]
pub fn reveal_project_file(mod_id: String, relative_path: String) -> Result<String, String> {
    project_files::reveal_project_file(mod_id, relative_path)
}

#[tauri::command]
pub fn create_expected_project_folder(
    mod_id: String,
    expected_path: String,
) -> Result<String, String> {
    project_files::create_expected_project_folder(mod_id, expected_path)
}

#[tauri::command]
pub fn get_asset_registry(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<AssetRegistryDto, String> {
    assets::get_asset_registry(session_id, sessions)
}

#[tauri::command]
pub fn create_asset_descriptor(
    app: AppHandle,
    session_id: String,
    request: CreateAssetDescriptorRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ManagedAssetDto, String> {
    assets::create_asset_descriptor(app, session_id, request, sessions)
}

#[tauri::command]
pub fn create_spritesheet_ruleset(
    app: AppHandle,
    session_id: String,
    request: CreateSpritesheetRulesetRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ManagedAssetDto, String> {
    assets::create_spritesheet_ruleset(app, session_id, request, sessions)
}

#[tauri::command]
pub fn load_sheet_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    sheets::load_sheet_resource(session_id, resource_uri, sessions)
}

#[tauri::command]
pub fn load_tilemap_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TilemapResourceDto, String> {
    sheets::load_tilemap_resource(session_id, resource_uri, sessions)
}

#[tauri::command]
pub fn load_tile_ruleset_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TileRulesetResourceDto, String> {
    sheets::load_tile_ruleset_resource(session_id, resource_uri, sessions)
}

#[tauri::command]
pub fn save_tilemap_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    tilemap: TilemapResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TilemapResourceDto, String> {
    sheets::save_tilemap_resource(app, session_id, resource_uri, tilemap, sessions)
}

#[tauri::command]
pub fn save_sheet_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    sheet: SheetResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    sheets::save_sheet_resource(app, session_id, resource_uri, sheet, sessions)
}

#[tauri::command]
pub fn get_theme_settings() -> Result<crate::settings::theme::ThemeSettingsDto, String> {
    settings::get_theme_settings()
}

#[tauri::command]
pub fn set_theme_settings(
    app: AppHandle,
    theme_id: String,
) -> Result<crate::settings::theme::ThemeSettingsDto, String> {
    settings::set_theme_settings(app, theme_id)
}

#[tauri::command]
pub fn set_font_settings(
    app: AppHandle,
    font_id: String,
) -> Result<crate::settings::theme::ThemeSettingsDto, String> {
    settings::set_font_settings(app, font_id)
}

#[tauri::command]
pub fn get_editor_settings() -> Result<EditorSettingsDto, String> {
    settings::get_editor_settings()
}

#[tauri::command]
pub fn set_editor_mods_root(mods_root: String) -> Result<EditorSettingsDto, String> {
    settings::set_editor_mods_root(mods_root)
}

#[tauri::command]
pub fn reset_editor_mods_root() -> Result<EditorSettingsDto, String> {
    settings::reset_editor_mods_root()
}

#[tauri::command]
pub fn pick_mods_root() -> Result<Option<String>, String> {
    settings::pick_mods_root()
}

#[tauri::command]
pub fn get_cache_info(paths: State<'_, EditorPaths>) -> Result<CacheInfoDto, String> {
    cache::get_cache_info(paths)
}

#[tauri::command]
pub fn get_cache_policy(paths: State<'_, EditorPaths>) -> Result<CachePolicyDto, String> {
    cache::get_cache_policy(paths)
}

#[tauri::command]
pub fn set_cache_policy(
    paths: State<'_, EditorPaths>,
    policy: CachePolicyDto,
) -> Result<CachePolicyDto, String> {
    cache::set_cache_policy(paths, policy)
}

#[tauri::command]
pub fn run_cache_maintenance(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    cache::run_cache_maintenance(app, paths)
}

#[tauri::command]
pub fn clear_orphaned_project_caches(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    cache::clear_orphaned_project_caches(app, paths)
}

#[tauri::command]
pub fn clear_project_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::clear_project_cache(app, paths, project_cache_id)
}

#[tauri::command]
pub fn clear_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::clear_preview_cache(app, paths, project_cache_id)
}

#[tauri::command]
pub fn clear_all_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<(), String> {
    cache::clear_all_preview_cache(app, paths)
}

#[tauri::command]
pub fn reveal_cache_folder(paths: State<'_, EditorPaths>) -> Result<String, String> {
    cache::reveal_cache_folder(paths)
}
