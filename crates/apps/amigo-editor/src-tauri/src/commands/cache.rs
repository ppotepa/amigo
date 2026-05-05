use tauri::{AppHandle, State};

use crate::cache as app_cache;
use crate::cache::index;
use crate::cache::root::EditorPaths;
use crate::dto::{CacheInfoDto, CacheMaintenanceResultDto, CachePolicyDto};
use crate::events::bus;

use super::shared::reveal_path;

pub fn get_cache_info(paths: State<'_, EditorPaths>) -> Result<CacheInfoDto, String> {
    index::collect_cache_info(&paths.cache_root, &paths.cache_root_mode)
}

pub fn get_cache_policy(paths: State<'_, EditorPaths>) -> Result<CachePolicyDto, String> {
    Ok(app_cache::policies::load_cache_policy(&paths.cache_root))
}

pub fn set_cache_policy(
    paths: State<'_, EditorPaths>,
    policy: CachePolicyDto,
) -> Result<CachePolicyDto, String> {
    app_cache::policies::save_cache_policy(&paths.cache_root, &policy)?;
    Ok(policy)
}

pub fn run_cache_maintenance(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    let result = app_cache::maintenance::run_cache_maintenance(&paths.cache_root)?;
    let _ = bus::emit_cache_invalidated(&app, None, None, None, None, "all", "maintenance");
    Ok(result)
}

pub fn clear_orphaned_project_caches(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    let result = app_cache::maintenance::clear_orphaned_project_caches(&paths.cache_root)?;
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        None,
        None,
        None,
        "project",
        "orphaned-project-caches-cleared",
    );
    Ok(result)
}

pub fn clear_project_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    app_cache::index::clear_project_cache(&paths.cache_root, &project_cache_id)?;
    bus::emit_cache_invalidated(
        &app,
        Some(project_cache_id),
        None,
        None,
        None,
        "project",
        "project-cache-cleared",
    )
}

pub fn clear_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    app_cache::index::clear_preview_cache(&paths.cache_root, &project_cache_id)?;
    bus::emit_cache_invalidated(
        &app,
        Some(project_cache_id),
        None,
        None,
        None,
        "preview",
        "preview-cache-cleared",
    )
}

pub fn clear_all_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<(), String> {
    let projects_path = app_cache::index::projects_dir(&paths.cache_root);
    if !projects_path.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&projects_path).map_err(|error| {
        format!(
            "failed to read project cache `{}`: {error}",
            projects_path.display()
        )
    })? {
        let entry =
            entry.map_err(|error| format!("failed to read project cache entry: {error}"))?;
        if entry.path().is_dir() {
            app_cache::index::clear_preview_cache(
                &paths.cache_root,
                &entry.file_name().to_string_lossy(),
            )?;
        }
    }
    bus::emit_cache_invalidated(
        &app,
        None,
        None,
        None,
        None,
        "preview",
        "all-preview-cache-cleared",
    )
}

pub fn reveal_cache_folder(paths: State<'_, EditorPaths>) -> Result<String, String> {
    reveal_path(&paths.cache_root)?;
    Ok(paths.cache_root.display().to_string())
}
