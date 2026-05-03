use std::path::Path;
use std::process::Command;

use rfd::FileDialog;
use tauri::{AppHandle, Emitter, State};

use crate::cache;
use crate::cache::index;
use crate::dto::{
    CacheInfoDto, CacheMaintenanceResultDto, CachePolicyDto, EditorModDetailsDto,
    EditorModSummaryDto, EditorSceneEntityDto, EditorSceneHierarchyDto, EditorSessionDto,
    EditorSettingsDto, OpenModResultDto,
    ScenePreviewDto, ScenePreviewFrameGeneratedDto,
};
use crate::mods::discovery::{discover_editor_mods, discovered_mod_ids};
use crate::mods::metadata::{mod_details, mod_summary};
use crate::preview::renderer;
use crate::settings::editor_settings::{
    load_editor_settings, save_editor_settings,
};
use crate::settings::theme::{validate_theme_id, ThemeSettingsDto};
use crate::{cache::root::EditorPaths, session::EditorSessionRegistry};

#[tauri::command]
pub fn list_known_mods(paths: State<'_, EditorPaths>) -> Result<Vec<EditorModSummaryDto>, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    for discovered_mod in &discovered {
        let _ = index::upsert_project_index_entry(&paths.cache_root, discovered_mod);
    }
    Ok(discovered
        .iter()
        .map(|discovered_mod| mod_summary(discovered_mod, &discovered_ids))
        .collect())
}

#[tauri::command]
pub fn get_mod_details(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let _ = index::upsert_project_index_entry(&paths.cache_root, discovered_mod);
    Ok(mod_details(discovered_mod, &discovered_ids))
}

#[tauri::command]
pub async fn request_scene_preview(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    mod_id: String,
    scene_id: String,
    force_regenerate: Option<bool>,
) -> Result<ScenePreviewDto, String> {
    let cache_root = paths.cache_root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
        let discovered_mod = discovered
            .iter()
            .find(|candidate| candidate.manifest.id == mod_id)
            .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
        renderer::request_scene_preview(
            discovered_mod,
            &scene_id,
            force_regenerate.unwrap_or(false),
            &cache_root,
            |current, total| {
                let _ = app.emit(
                    "scene-preview-frame-generated",
                    ScenePreviewFrameGeneratedDto {
                        mod_id: mod_id.clone(),
                        scene_id: scene_id.clone(),
                        current,
                        total,
                    },
                );
            },
        )
    })
    .await
    .map_err(|error| format!("preview task failed to join: {error}"))?
}

#[tauri::command]
pub fn open_mod(
    mod_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<OpenModResultDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let selected_scene_id = discovered_mod
        .manifest
        .scenes
        .iter()
        .find(|scene| scene.is_launcher_visible())
        .or_else(|| discovered_mod.manifest.scenes.first())
        .map(|scene| scene.id.clone());

    let mut settings = load_editor_settings();
    settings.last_opened_mod_id = Some(mod_id.clone());
    let _ = save_editor_settings(&settings);

    let session = sessions.create_session(
        mod_id.clone(),
        discovered_mod.root_path.display().to_string(),
        selected_scene_id,
    )?;

    Ok(OpenModResultDto {
        mod_id,
        root_path: discovered_mod.root_path.display().to_string(),
        session_id: session.session_id,
        created_at: session.created_at,
        selected_scene_id: session.selected_scene_id,
    })
}

#[tauri::command]
pub fn get_editor_session(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorSessionDto, String> {
    sessions.get_session(&session_id)
}

#[tauri::command]
pub fn close_editor_session(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<(), String> {
    sessions.close_session(&session_id)
}

#[tauri::command]
pub fn validate_mod(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    get_mod_details(mod_id, paths)
}

#[tauri::command]
pub async fn regenerate_all_scene_previews(
    paths: State<'_, EditorPaths>,
    mod_id: String,
) -> Result<Vec<ScenePreviewDto>, String> {
    let cache_root = paths.cache_root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
        let discovered_mod = discovered
            .iter()
            .find(|candidate| candidate.manifest.id == mod_id)
            .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

        discovered_mod
            .manifest
            .scenes
            .iter()
            .map(|scene| {
                renderer::request_scene_preview(
                    discovered_mod,
                    &scene.id,
                    true,
                    &cache_root,
                    |_, _| {},
                )
            })
            .collect()
    })
    .await
    .map_err(|error| format!("preview task failed to join: {error}"))?
}

#[tauri::command]
pub fn reveal_mod_folder(mod_id: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    reveal_path(&discovered_mod.root_path)?;
    Ok(discovered_mod.root_path.display().to_string())
}

#[tauri::command]
pub fn reveal_scene_document(mod_id: String, scene_id: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let document_path = discovered_mod
        .scene_document_path(&scene_id)
        .ok_or_else(|| format!("scene `{scene_id}` was not found in mod `{mod_id}`"))?;
    reveal_path(&document_path)?;
    Ok(document_path.display().to_string())
}

#[tauri::command]
pub fn get_scene_hierarchy(
    mod_id: String,
    scene_id: String,
) -> Result<EditorSceneHierarchyDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let document_path = discovered_mod
        .scene_document_path(&scene_id)
        .ok_or_else(|| format!("scene `{scene_id}` was not found in mod `{mod_id}`"))?;
    let document = amigo_scene::load_scene_document_from_path(&document_path)
        .map_err(|error| format!("failed to load scene document `{}`: {error}", document_path.display()))?;

    let entities = document
        .entities
        .iter()
        .map(|entity| EditorSceneEntityDto {
            id: entity.id.clone(),
            name: entity.display_name(),
            tags: entity.tags.clone(),
            groups: entity.groups.clone(),
            visible: entity.visible,
            simulation_enabled: entity.simulation_enabled,
            collision_enabled: entity.collision_enabled,
            has_transform2: entity.transform2.is_some(),
            has_transform3: entity.transform3.is_some(),
            property_count: entity.properties.len(),
            component_count: entity.components.len(),
            component_types: entity
                .components
                .iter()
                .map(|component| component.kind().to_owned())
                .collect(),
        })
        .collect::<Vec<_>>();
    let component_count = entities.iter().map(|entity| entity.component_count).sum();

    Ok(EditorSceneHierarchyDto {
        mod_id,
        scene_id,
        scene_label: document.scene.label,
        entity_count: entities.len(),
        component_count,
        entities,
        diagnostics: Vec::new(),
    })
}

#[tauri::command]
pub fn get_theme_settings() -> Result<ThemeSettingsDto, String> {
    Ok(ThemeSettingsDto {
        active_theme_id: load_editor_settings().active_theme_id,
    })
}

#[tauri::command]
pub fn set_theme_settings(theme_id: String) -> Result<ThemeSettingsDto, String> {
    validate_theme_id(&theme_id)?;
    let mut settings = load_editor_settings();
    settings.active_theme_id = theme_id.clone();
    save_editor_settings(&settings).map_err(|error| format!("failed to persist theme settings: {error}"))?;
    Ok(ThemeSettingsDto {
        active_theme_id: settings.active_theme_id,
    })
}

#[tauri::command]
pub fn get_editor_settings() -> Result<EditorSettingsDto, String> {
    Ok(load_editor_settings())
}

#[tauri::command]
pub fn set_editor_mods_root(mods_root: String) -> Result<EditorSettingsDto, String> {
    let mut settings = load_editor_settings();
    settings.mods_root = Some(mods_root);
    save_editor_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn reset_editor_mods_root() -> Result<EditorSettingsDto, String> {
    let mut settings = load_editor_settings();
    settings.mods_root = None;
    save_editor_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn pick_mods_root() -> Result<Option<String>, String> {
    let folder = FileDialog::new().set_title("Choose Mods Root").pick_folder();
    Ok(folder.map(|path| path.display().to_string()))
}

#[tauri::command]
pub fn get_cache_info(paths: State<'_, EditorPaths>) -> Result<CacheInfoDto, String> {
    index::collect_cache_info(&paths.cache_root, &paths.cache_root_mode)
}

#[tauri::command]
pub fn get_cache_policy(paths: State<'_, EditorPaths>) -> Result<CachePolicyDto, String> {
    Ok(cache::policies::load_cache_policy(&paths.cache_root))
}

#[tauri::command]
pub fn set_cache_policy(
    paths: State<'_, EditorPaths>,
    policy: CachePolicyDto,
) -> Result<CachePolicyDto, String> {
    cache::policies::save_cache_policy(&paths.cache_root, &policy)?;
    Ok(policy)
}

#[tauri::command]
pub fn run_cache_maintenance(
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    cache::maintenance::run_cache_maintenance(&paths.cache_root)
}

#[tauri::command]
pub fn clear_orphaned_project_caches(
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    cache::maintenance::clear_orphaned_project_caches(&paths.cache_root)
}

#[tauri::command]
pub fn clear_project_cache(
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::index::clear_project_cache(&paths.cache_root, &project_cache_id)
}

#[tauri::command]
pub fn clear_preview_cache(
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::index::clear_preview_cache(&paths.cache_root, &project_cache_id)
}

#[tauri::command]
pub fn clear_all_preview_cache(paths: State<'_, EditorPaths>) -> Result<(), String> {
    let projects_path = cache::index::projects_dir(&paths.cache_root);
    if !projects_path.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&projects_path)
        .map_err(|error| format!("failed to read project cache `{}`: {error}", projects_path.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read project cache entry: {error}"))?;
        if entry.path().is_dir() {
            cache::index::clear_preview_cache(&paths.cache_root, &entry.file_name().to_string_lossy())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn reveal_cache_folder(paths: State<'_, EditorPaths>) -> Result<String, String> {
    reveal_path(&paths.cache_root)?;
    Ok(paths.cache_root.display().to_string())
}

fn reveal_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = if path.is_file() {
            Command::new("explorer")
                .arg(format!("/select,{}", path.display()))
                .status()
        } else {
            Command::new("explorer").arg(path).status()
        }
        .map_err(|error| error.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("explorer failed for `{}`", path.display()))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        let status = Command::new(opener)
            .arg(target)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("{opener} failed for `{}`", target.display()))
        }
    }
}
