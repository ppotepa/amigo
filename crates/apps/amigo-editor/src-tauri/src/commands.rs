use std::path::{Path, PathBuf};
use std::process::Command;

use rfd::FileDialog;
use tauri::{AppHandle, Manager, State};

use crate::asset_registry::dto::{
    AssetMigrationPlanDto, AssetMigrationResultDto, AssetRegistryDto,
    CreateAssetDescriptorRequestDto, ManagedAssetDto,
};
use crate::cache;
use crate::cache::index;
use crate::dto::{
    CacheInfoDto, CacheMaintenanceResultDto, CachePolicyDto, EditorModDetailsDto,
    EditorModSummaryDto, EditorProjectFileContentDto, EditorProjectFileDto,
    EditorProjectStructureNodeDto, EditorProjectStructureTreeDto, EditorProjectTreeDto,
    EditorSceneEntityDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, EditorSessionDto,
    EditorSettingsDto, EditorWindowRegistryDto, OpenModResultDto, ScenePreviewDto,
    ScenePreviewFrameGeneratedDto, WriteProjectFileRequestDto,
};
use crate::events::bus;
use crate::events::preview_progress::{PreviewProgressPayload, emit_preview_progress};
use crate::mods::discovery::{discover_editor_mods, discovered_mod_ids};
use crate::mods::metadata::{mod_details, mod_summary};
use crate::preview::renderer;
use crate::settings::editor_settings::{load_editor_settings, save_editor_settings};
use crate::settings::theme::{
    ThemeSettingsDto, normalize_font_id, normalize_theme_id, validate_font_id, validate_theme_id,
};
use crate::sheet::dto::SheetResourceDto;
use crate::windows::commands::{
    open_mod_settings_window as open_mod_settings_window_impl,
    open_settings_window as open_settings_window_impl, open_theme_window as open_theme_window_impl,
};
use crate::windows::descriptors::EditorWindowKind;
use crate::windows::manager::open_or_focus_window;
use crate::windows::registry::EditorWindowRegistry;
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
    let force_regenerate = force_regenerate.unwrap_or(false);
    let progress_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
        let discovered_mod = discovered
            .iter()
            .find(|candidate| candidate.manifest.id == mod_id)
            .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
        let project_cache_id =
            crate::cache::project_id::project_cache_id_for_root(&discovered_mod.root_path);
        renderer::request_scene_preview(
            discovered_mod,
            &scene_id,
            force_regenerate,
            &cache_root,
            |current, total| {
                let _ = emit_preview_progress(
                    &progress_app,
                    PreviewProgressPayload::from(ScenePreviewFrameGeneratedDto {
                        mod_id: mod_id.clone(),
                        scene_id: scene_id.clone(),
                        current,
                        total,
                    }),
                );
            },
        )
        .map(|preview| (preview, project_cache_id))
    })
    .await
    .map_err(|error| format!("preview task failed to join: {error}"))
    .and_then(|result| {
        let (preview, project_cache_id) = result?;
        if force_regenerate {
            let _ = bus::emit_cache_invalidated(
                &app,
                Some(project_cache_id),
                Some(preview.mod_id.clone()),
                Some(preview.scene_id.clone()),
                Some(preview.source_hash.clone()),
                "preview",
                "scene-preview-regenerated",
            );
        }
        Ok(preview)
    })
}

#[tauri::command]
pub fn open_mod(
    mod_id: String,
    selected_scene_id: Option<String>,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<OpenModResultDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let selected_scene_id = selected_scene_id.or_else(|| {
        discovered_mod
            .manifest
            .scenes
            .iter()
            .find(|scene| scene.is_launcher_visible())
            .or_else(|| discovered_mod.manifest.scenes.first())
            .map(|scene| scene.id.clone())
    });

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
pub fn open_mod_workspace(
    app: AppHandle,
    mod_id: String,
    selected_scene_id: Option<String>,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<OpenModResultDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let selected_scene_id = selected_scene_id.or_else(|| {
        discovered_mod
            .manifest
            .scenes
            .iter()
            .find(|scene| scene.is_launcher_visible())
            .or_else(|| discovered_mod.manifest.scenes.first())
            .map(|scene| scene.id.clone())
    });

    let mut settings = load_editor_settings();
    settings.last_opened_mod_id = Some(mod_id.clone());
    let _ = save_editor_settings(&settings);

    let session = sessions.create_session(
        mod_id.clone(),
        discovered_mod.root_path.display().to_string(),
        selected_scene_id,
    )?;
    open_or_focus_window(
        &app,
        EditorWindowKind::Workspace {
            session_id: session.session_id.clone(),
            title: discovered_mod.manifest.name.clone(),
        },
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
    sessions.get_session(&session_id)
}

#[tauri::command]
pub fn close_editor_session(
    app: AppHandle,
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<(), String> {
    sessions.close_session(&session_id)?;
    bus::emit_session_closed(&app, session_id)
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
    let document = amigo_scene::load_scene_document_from_path(&document_path).map_err(|error| {
        format!(
            "failed to load scene document `{}`: {error}",
            document_path.display()
        )
    })?;

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
pub fn get_project_tree(mod_id: String) -> Result<EditorProjectTreeDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let mut total_files = 0;
    let root = project_file_node(
        &discovered_mod.root_path,
        &discovered_mod.root_path,
        &mut total_files,
    )?;

    Ok(EditorProjectTreeDto {
        mod_id,
        root_path: discovered_mod.root_path.display().to_string(),
        total_files,
        root,
    })
}

#[tauri::command]
pub fn get_project_structure_tree(mod_id: String) -> Result<EditorProjectStructureTreeDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let mut total_files = 0;
    let file_root = project_file_node(
        &discovered_mod.root_path,
        &discovered_mod.root_path,
        &mut total_files,
    )?;
    let details = mod_details(discovered_mod, &discovered_ids);

    Ok(EditorProjectStructureTreeDto {
        mod_id,
        root_path: discovered_mod.root_path.display().to_string(),
        root: project_structure_root(&details, &file_root),
    })
}

#[tauri::command]
pub fn read_project_file(
    mod_id: String,
    relative_path: String,
) -> Result<EditorProjectFileContentDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &relative_path)?;
    let metadata = std::fs::metadata(&path)
        .map_err(|error| format!("failed to read metadata `{}`: {error}", path.display()))?;

    if !metadata.is_file() {
        return Err(format!("project path `{relative_path}` is not a file"));
    }

    let kind = classify_project_file(&path, false);
    if !is_readable_project_text_kind(&kind) {
        return Err(format!(
            "project file `{relative_path}` is not a supported text file"
        ));
    }

    const MAX_TEXT_FILE_BYTES: u64 = 512 * 1024;
    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(format!(
            "project file `{relative_path}` is too large to preview as text: {} bytes",
            metadata.len()
        ));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read project file `{}`: {error}", path.display()))?;

    Ok(EditorProjectFileContentDto {
        mod_id,
        path: path.display().to_string(),
        relative_path,
        language: language_for_project_file_kind(&kind).to_owned(),
        kind,
        size_bytes: metadata.len(),
        content,
        diagnostics: Vec::new(),
    })
}

#[tauri::command]
pub fn write_project_file(
    app: AppHandle,
    mod_id: String,
    request: WriteProjectFileRequestDto,
) -> Result<EditorProjectFileContentDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &request.relative_path)?;
    let kind = classify_project_file(&path, false);
    if !is_readable_project_text_kind(&kind) {
        return Err(format!(
            "project file `{}` is not a supported text file",
            request.relative_path
        ));
    }
    std::fs::write(&path, request.content.as_bytes())
        .map_err(|error| format!("failed to write project file `{}`: {error}", path.display()))?;

    let relative_path = request.relative_path.clone();
    if relative_path.ends_with(".image.yml")
        || relative_path.ends_with(".sprite.yml")
        || relative_path.ends_with(".atlas.yml")
        || relative_path.ends_with(".tileset.yml")
        || relative_path.ends_with(".tilemap.yml")
    {
        let asset_key = asset_key_from_descriptor_relative_path(&mod_id, &relative_path)
            .unwrap_or_else(|| relative_path.clone());
        let _ = bus::emit_asset_descriptor_changed(
            &app,
            mod_id.clone(),
            asset_key,
            relative_path.clone(),
            "saved",
        );
        let _ = bus::emit_asset_registry_changed(&app, mod_id.clone());
        let _ = bus::emit_cache_invalidated(
            &app,
            None,
            Some(mod_id.clone()),
            None,
            None,
            "asset",
            "project text file saved",
        );
    }

    read_project_file(mod_id, relative_path)
}

#[tauri::command]
pub fn reveal_project_file(mod_id: String, relative_path: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &relative_path)?;
    reveal_path(&path)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn create_expected_project_folder(
    mod_id: String,
    expected_path: String,
) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let normalized = expected_path.trim().replace('\\', "/");
    let allowed = [
        "scenes/",
        "assets/",
        "assets/raw/",
        "assets/raw/images/",
        "assets/raw/audio/",
        "assets/raw/fonts/",
        "assets/raw/other/",
        "assets/images/",
        "assets/sprites/",
        "assets/tilemaps/",
        "assets/tilesets/",
        "assets/fonts/",
        "assets/audio/",
        "assets/particles/",
        "assets/materials/",
        "assets/ui/",
        "scripts/",
        "packages/",
    ];
    if !allowed.contains(&normalized.as_str()) {
        return Err(format!(
            "expected project folder `{expected_path}` is not part of the current editor structure contract"
        ));
    }

    let relative_path = normalized.trim_end_matches('/');
    let target = discovered_mod.root_path.join(relative_path);
    let canonical_root = discovered_mod.root_path.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            discovered_mod.root_path.display()
        )
    })?;
    let parent = target
        .parent()
        .ok_or_else(|| format!("project folder `{expected_path}` has no parent"))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(format!("project folder `{expected_path}` escapes mod root"));
    }

    std::fs::create_dir_all(&target).map_err(|error| {
        format!(
            "failed to create project folder `{}`: {error}",
            target.display()
        )
    })?;
    Ok(target.display().to_string())
}

#[tauri::command]
pub fn get_asset_registry(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<AssetRegistryDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::asset_registry::scanner::scan_asset_registry(
        &session.session_id,
        &session.mod_id,
        Path::new(&session.root_path),
    )
}

#[tauri::command]
pub fn create_asset_descriptor(
    app: AppHandle,
    session_id: String,
    request: CreateAssetDescriptorRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ManagedAssetDto, String> {
    let session = sessions.get_session(&session_id)?;
    let asset = crate::asset_registry::scanner::create_asset_descriptor(
        &session.mod_id,
        Path::new(&session.root_path),
        request,
    )?;
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset.asset_key.clone(),
        asset.descriptor_relative_path.clone(),
        "created",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "asset",
        "asset descriptor created",
    );
    Ok(asset)
}

#[tauri::command]
pub fn scan_asset_migration_plan(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<AssetMigrationPlanDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::asset_registry::scanner::scan_asset_migration_plan(
        &session.session_id,
        &session.mod_id,
        Path::new(&session.root_path),
    )
}

#[tauri::command]
pub fn apply_asset_migration_plan(
    app: AppHandle,
    session_id: String,
    plan: AssetMigrationPlanDto,
    dry_run: bool,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<AssetMigrationResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    let result = crate::asset_registry::scanner::apply_asset_migration_plan(
        Path::new(&session.root_path),
        plan,
        dry_run,
    )?;
    if !dry_run {
        let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
        let _ = bus::emit_cache_invalidated(
            &app,
            None,
            Some(session.mod_id.clone()),
            None,
            None,
            "asset",
            "asset migration applied",
        );
    }
    Ok(result)
}

#[tauri::command]
pub fn load_sheet_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::sheet::loader::load_sheet_resource(Path::new(&session.root_path), &resource_uri)
}

#[tauri::command]
pub fn load_tilemap_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<crate::sheet::dto::TilemapResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::sheet::loader::load_tilemap_resource(Path::new(&session.root_path), &resource_uri)
}

#[tauri::command]
pub fn save_tilemap_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    tilemap: crate::sheet::dto::TilemapResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<crate::sheet::dto::TilemapResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    let saved = crate::sheet::loader::save_tilemap_resource(
        Path::new(&session.root_path),
        &resource_uri,
        tilemap,
    )?;
    let asset_key = asset_key_from_descriptor_relative_path(&session.mod_id, &saved.relative_path)
        .unwrap_or_else(|| saved.relative_path.clone());
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset_key,
        saved.relative_path.clone(),
        "saved",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "tilemap",
        "tilemap resource saved",
    );
    Ok(saved)
}

#[tauri::command]
pub fn save_sheet_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    sheet: SheetResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    let saved =
        crate::sheet::loader::save_sheet_resource(Path::new(&session.root_path), &resource_uri, sheet)?;
    let asset_key = asset_key_from_descriptor_relative_path(&session.mod_id, &saved.relative_path)
        .unwrap_or_else(|| saved.relative_path.clone());
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset_key,
        saved.relative_path.clone(),
        "saved",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "sheet",
        "sheet resource saved",
    );
    Ok(saved)
}

#[tauri::command]
pub fn get_theme_settings() -> Result<ThemeSettingsDto, String> {
    let settings = load_editor_settings();
    Ok(ThemeSettingsDto {
        active_theme_id: normalize_theme_id(&settings.active_theme_id)
            .unwrap_or("mexico-at-night")
            .to_owned(),
        active_font_id: normalize_font_id(&settings.active_font_id)
            .unwrap_or("source-sans-3")
            .to_owned(),
    })
}

#[tauri::command]
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

#[tauri::command]
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
    let folder = FileDialog::new()
        .set_title("Choose Mods Root")
        .pick_folder();
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
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    let result = cache::maintenance::run_cache_maintenance(&paths.cache_root)?;
    let _ = bus::emit_cache_invalidated(&app, None, None, None, None, "all", "maintenance");
    Ok(result)
}

#[tauri::command]
pub fn clear_orphaned_project_caches(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<CacheMaintenanceResultDto, String> {
    let result = cache::maintenance::clear_orphaned_project_caches(&paths.cache_root)?;
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

#[tauri::command]
pub fn clear_project_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::index::clear_project_cache(&paths.cache_root, &project_cache_id)?;
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

#[tauri::command]
pub fn clear_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    project_cache_id: String,
) -> Result<(), String> {
    cache::index::clear_preview_cache(&paths.cache_root, &project_cache_id)?;
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

#[tauri::command]
pub fn clear_all_preview_cache(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
) -> Result<(), String> {
    let projects_path = cache::index::projects_dir(&paths.cache_root);
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
            cache::index::clear_preview_cache(
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

#[tauri::command]
pub fn reveal_cache_folder(paths: State<'_, EditorPaths>) -> Result<String, String> {
    reveal_path(&paths.cache_root)?;
    Ok(paths.cache_root.display().to_string())
}

fn resolve_project_relative_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let candidate = root.join(relative_path);
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize project path `{}`: {error}",
            candidate.display()
        )
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!("project path `{relative_path}` escapes mod root"));
    }

    Ok(canonical_candidate)
}

fn project_file_node(
    path: &Path,
    root: &Path,
    total_files: &mut usize,
) -> Result<EditorProjectFileDto, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("failed to read metadata `{}`: {error}", path.display()))?;
    let is_dir = metadata.is_dir();
    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let name = if relative_path.is_empty() {
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("mod root")
            .to_owned()
    } else {
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned()
    };

    let mut children = Vec::new();
    if is_dir {
        let mut entries = std::fs::read_dir(path)
            .map_err(|error| format!("failed to read directory `{}`: {error}", path.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|entry_path| should_include_project_path(entry_path))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            let left_is_dir = left.is_dir();
            let right_is_dir = right.is_dir();
            right_is_dir
                .cmp(&left_is_dir)
                .then_with(|| left.file_name().cmp(&right.file_name()))
        });

        for entry_path in entries {
            children.push(project_file_node(&entry_path, root, total_files)?);
        }
    } else {
        *total_files += 1;
    }

    Ok(EditorProjectFileDto {
        name,
        path: path.display().to_string(),
        relative_path,
        kind: classify_project_file(path, is_dir),
        is_dir,
        size_bytes: if is_dir { 0 } else { metadata.len() },
        children,
    })
}

fn project_structure_root(
    details: &EditorModDetailsDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let summary = &details.summary.content_summary;
    let diagnostics_count = details.summary.diagnostics.len()
        + details
            .scenes
            .iter()
            .map(|scene| scene.diagnostics.len())
            .sum::<usize>();
    let script_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| {
            file.kind == "script" && !scene_owns_script(&details.scenes, &file.relative_path)
        })
        .cloned()
        .collect::<Vec<_>>();
    let package_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| file.relative_path.starts_with("packages/"))
        .cloned()
        .collect::<Vec<_>>();

    node(ProjectStructureNodeInput {
        id: format!("mod:{}", details.summary.id),
        label: details.summary.id.clone(),
        kind: "modRoot",
        icon: "Mod",
        status: Some(project_status_for_editor_status(&format!(
            "{:?}",
            details.summary.status
        ))),
        count: Some(summary.total_files),
        path: Some(details.summary.root_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: None,
        scene: None,
        children: vec![
            node(ProjectStructureNodeInput {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                kind: "overview",
                icon: "Info",
                status: Some(project_status_for_editor_status(&format!(
                    "{:?}",
                    details.summary.status
                ))),
                count: None,
                path: None,
                expected_path: None,
                exists: true,
                empty: false,
                ghost: false,
                file: None,
                scene: None,
                children: Vec::new(),
            }),
            manifest_node(file_root, details),
            group_node(
                "scenes",
                "Sc",
                details.scenes.len(),
                root_child_exists(file_root, "scenes"),
                details
                    .scenes
                    .iter()
                    .map(|scene| scene_structure_node(scene, file_root))
                    .collect(),
            ),
            group_node(
                "assets",
                "Assets",
                summary.textures
                    + summary.spritesheets
                    + summary.tilemaps
                    + summary.tilesets
                    + summary.audio
                    + summary.fonts
                    + summary.unknown_files,
                root_child_exists(file_root, "assets"),
                asset_category_nodes(summary, file_root),
            ),
            group_node(
                "scripts",
                "Rh",
                script_files.len(),
                root_child_exists(file_root, "scripts"),
                script_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptFile"))
                    .collect(),
            ),
            group_node(
                "packages",
                "Pkg",
                summary.packages,
                root_child_exists(file_root, "packages"),
                package_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptPackage"))
                    .collect(),
            ),
            virtual_node(
                "capabilities",
                "Capabilities",
                "Plug",
                details.summary.capabilities.len(),
                "ok",
            ),
            virtual_node(
                "dependencies",
                "Dependencies",
                "Link",
                details.summary.dependencies.len(),
                if details.summary.missing_dependencies.is_empty() {
                    "ok"
                } else {
                    "warn"
                },
            ),
            virtual_node(
                "diagnostics",
                "Diagnostics",
                "Diag",
                diagnostics_count,
                if diagnostics_count == 0 { "ok" } else { "warn" },
            ),
        ],
    })
}

struct ProjectStructureNodeInput {
    id: String,
    label: String,
    kind: &'static str,
    icon: &'static str,
    status: Option<String>,
    count: Option<usize>,
    path: Option<String>,
    expected_path: Option<String>,
    exists: bool,
    empty: bool,
    ghost: bool,
    file: Option<EditorProjectFileDto>,
    scene: Option<EditorSceneSummaryDto>,
    children: Vec<EditorProjectStructureNodeDto>,
}

fn node(input: ProjectStructureNodeInput) -> EditorProjectStructureNodeDto {
    EditorProjectStructureNodeDto {
        id: input.id,
        label: input.label,
        kind: input.kind.to_owned(),
        icon: input.icon.to_owned(),
        status: input.status,
        count: input.count,
        path: input.path,
        expected_path: input.expected_path,
        exists: input.exists,
        empty: input.empty,
        ghost: input.ghost,
        file: input.file,
        scene: input.scene,
        children: input.children,
    }
}

fn manifest_node(
    file_root: &EditorProjectFileDto,
    details: &EditorModDetailsDto,
) -> EditorProjectStructureNodeDto {
    let manifest = find_project_file(file_root, "mod.toml").cloned();
    node(ProjectStructureNodeInput {
        id: "manifest:mod.toml".to_owned(),
        label: "mod.toml".to_owned(),
        kind: "manifest",
        icon: "Toml",
        status: Some(if manifest.is_some() {
            project_status_for_editor_status(&format!("{:?}", details.summary.status))
        } else {
            "error".to_owned()
        }),
        count: None,
        path: manifest.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some("mod.toml".to_owned()),
        exists: manifest.is_some(),
        empty: false,
        ghost: manifest.is_none(),
        file: manifest,
        scene: None,
        children: Vec::new(),
    })
}

fn group_node(
    label: &str,
    icon: &'static str,
    count: usize,
    exists: bool,
    children: Vec<EditorProjectStructureNodeDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("group:{label}"),
        label: label.to_owned(),
        kind: if exists { "folder" } else { "expectedFolder" },
        icon,
        status: Some(
            if exists {
                if count == 0 { "empty" } else { "ok" }
            } else {
                "missing"
            }
            .to_owned(),
        ),
        count: Some(count),
        path: if exists { Some(label.to_owned()) } else { None },
        expected_path: Some(format!("{label}/")),
        exists,
        empty: exists && count == 0,
        ghost: !exists,
        file: None,
        scene: None,
        children,
    })
}

fn virtual_node(
    id: &'static str,
    label: &str,
    icon: &'static str,
    count: usize,
    status: &str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("virtual:{id}"),
        label: label.to_owned(),
        kind: id,
        icon,
        status: Some(status.to_owned()),
        count: Some(count),
        path: None,
        expected_path: None,
        exists: true,
        empty: count == 0,
        ghost: false,
        file: None,
        scene: None,
        children: Vec::new(),
    })
}

fn scene_structure_node(
    scene: &EditorSceneSummaryDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let document_path = relative_project_path(&scene.document_path);
    let script_path = relative_project_path(&scene.script_path);
    let document = find_project_file(file_root, &document_path).cloned();
    let script = find_project_file(file_root, &script_path).cloned();
    let status = project_status_for_editor_status(&format!("{:?}", scene.status));

    node(ProjectStructureNodeInput {
        id: format!("scene:{}", scene.id),
        label: if scene.label.is_empty() {
            scene.id.clone()
        } else {
            scene.label.clone()
        },
        kind: "scene",
        icon: "Play",
        status: Some(if status == "valid" {
            "ready".to_owned()
        } else {
            status
        }),
        count: Some(2),
        path: Some(scene.path.clone()),
        expected_path: None,
        exists: document.is_some(),
        empty: false,
        ghost: false,
        file: None,
        scene: Some(scene.clone()),
        children: vec![
            scene_file_node(
                "sceneDocument",
                format!("scene-doc:{}", scene.id),
                "scene.yml",
                "Yml",
                document_path,
                document,
            ),
            scene_file_node(
                "sceneScript",
                format!("scene-script:{}", scene.id),
                "scene.rhai",
                "Rh",
                script_path,
                script,
            ),
        ],
    })
}

fn scene_file_node(
    kind: &'static str,
    id: String,
    label: &str,
    icon: &'static str,
    expected_path: String,
    file: Option<EditorProjectFileDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id,
        label: label.to_owned(),
        kind,
        icon,
        status: Some(if file.is_some() { "ok" } else { "missing" }.to_owned()),
        count: None,
        path: file.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some(expected_path),
        exists: file.is_some(),
        empty: false,
        ghost: file.is_none(),
        file,
        scene: None,
        children: Vec::new(),
    })
}

fn asset_category_nodes(
    summary: &crate::dto::EditorContentSummaryDto,
    file_root: &EditorProjectFileDto,
) -> Vec<EditorProjectStructureNodeDto> {
    [
        ("images", "Img", summary.textures),
        ("sprites", "Grid", summary.spritesheets),
        ("tilesets", "Tile", summary.tilesets),
        ("tilemaps", "Map", summary.tilemaps),
        ("fonts", "Type", summary.fonts),
        ("audio", "Aud", summary.audio),
        ("particles", "Pt", 0),
        ("materials", "Mat", 0),
        ("ui", "Ui", 0),
    ]
    .into_iter()
    .map(|(label, icon, count)| {
        let expected_path = Some(format!("assets/{label}/"));
        let children = asset_category_files(file_root, label)
            .into_iter()
            .map(|file| asset_resource_node(file))
            .collect::<Vec<_>>();
        let actual_path = asset_category_path(file_root, label);
        let count = children.len().max(count);
        let exists = actual_path.is_some();
        node(ProjectStructureNodeInput {
            id: format!("asset:{label}"),
            label: label.to_owned(),
            kind: if exists {
                "assetCategory"
            } else {
                "expectedFolder"
            },
            icon,
            status: Some(
                if exists {
                    if count == 0 { "empty" } else { "ok" }
                } else {
                    "missing"
                }
                .to_owned(),
            ),
            count: Some(count),
            path: actual_path,
            expected_path,
            exists,
            empty: exists && count == 0,
            ghost: !exists,
            file: None,
            scene: None,
            children,
        })
    })
    .collect()
}

fn asset_category_files(root: &EditorProjectFileDto, label: &str) -> Vec<EditorProjectFileDto> {
    flatten_project_files(root)
        .into_iter()
        .filter(|file| asset_category_matches_file(label, file))
        .cloned()
        .collect()
}

fn asset_category_matches_file(label: &str, file: &EditorProjectFileDto) -> bool {
    match label {
        "images" => file.kind == "imageAsset",
        "sprites" => file.kind == "spritesheet",
        "tilemaps" => file.kind == "tilemap",
        "tilesets" => file.kind == "tileset",
        "audio" => file.kind == "audio",
        "fonts" => file.kind == "font",
        "particles" => file.kind == "particle",
        "materials" => file.kind == "material",
        "ui" => file.kind == "ui",
        _ => false,
    }
}

fn asset_category_path(root: &EditorProjectFileDto, label: &str) -> Option<String> {
    let preferred = format!("assets/{label}");
    if root_child_exists(root, &preferred) {
        return Some(preferred);
    }

    if root_child_exists(root, label) {
        return Some(label.to_owned());
    }

    None
}

fn file_structure_node(
    file: EditorProjectFileDto,
    kind: &'static str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("{kind}:{}", file.relative_path),
        label: file.name.clone(),
        kind,
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn asset_resource_node(file: EditorProjectFileDto) -> EditorProjectStructureNodeDto {
    let label = asset_display_label(&file);
    node(ProjectStructureNodeInput {
        id: format!("assetResource:{}", file.relative_path),
        label,
        kind: "assetResource",
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn flatten_project_files(root: &EditorProjectFileDto) -> Vec<&EditorProjectFileDto> {
    root.children
        .iter()
        .flat_map(|child| {
            let mut files = vec![child];
            files.extend(flatten_project_files(child));
            files
        })
        .filter(|file| !file.is_dir)
        .collect()
}

fn find_project_file<'a>(
    root: &'a EditorProjectFileDto,
    relative_path: &str,
) -> Option<&'a EditorProjectFileDto> {
    if root.relative_path == relative_path {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| find_project_file(child, relative_path))
}

fn root_child_exists(root: &EditorProjectFileDto, relative_path: &str) -> bool {
    find_project_file(root, relative_path).is_some()
}

fn relative_project_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    for prefix in ["scenes/", "scripts/", "assets/", "packages/"] {
        if let Some(index) = normalized.find(prefix) {
            return normalized[index..].to_owned();
        }
    }
    normalized
}

fn project_status_for_editor_status(status: &str) -> String {
    match status {
        "Valid" => "valid",
        "Warning" | "MissingDependency" => "warn",
        "Error" | "InvalidManifest" | "MissingSceneFile" | "PreviewFailed" => "error",
        _ => "ok",
    }
    .to_owned()
}

fn project_file_icon(file: &EditorProjectFileDto) -> &'static str {
    match file.kind.as_str() {
        "manifest" => "Toml",
        "sceneDocument" => "Yml",
        "script" => "Rh",
        "imageAsset" | "rawImage" | "texture" => "Img",
        "spritesheet" => "Grid",
        "audio" | "rawAudio" => "Aud",
        "font" | "rawFont" => "Type",
        "tilemap" => "Map",
        "tileset" => "Tile",
        "particle" => "Pt",
        "material" => "Mat",
        "ui" => "Ui",
        _ => "F",
    }
}

fn asset_display_label(file: &EditorProjectFileDto) -> String {
    let name = file.name.as_str();
    for suffix in [
        ".image.yml",
        ".image.yaml",
        ".sprite.yml",
        ".sprite.yaml",
        ".atlas.yml",
        ".atlas.yaml",
        ".tileset.yml",
        ".tileset.yaml",
        ".tile-ruleset.yml",
        ".tile-ruleset.yaml",
        ".tilemap.yml",
        ".tilemap.yaml",
        ".font.yml",
        ".font.yaml",
        ".audio.yml",
        ".audio.yaml",
        ".particle.yml",
        ".particle.yaml",
        ".material.yml",
        ".material.yaml",
        ".ui.yml",
        ".ui.yaml",
    ] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_owned();
        }
    }
    name.to_owned()
}

fn scene_owns_script(scenes: &[EditorSceneSummaryDto], relative_path: &str) -> bool {
    scenes
        .iter()
        .any(|scene| relative_project_path(&scene.script_path) == relative_path)
}

fn should_include_project_path(path: &PathBuf) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    !matches!(name, ".git" | ".amigo-editor" | "target")
}

fn classify_project_file(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "directory".to_owned();
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name == "mod.toml" || extension == "toml" {
        "manifest"
    } else if file_name == "package.yml" || file_name == "package.yaml" {
        "scriptPackage"
    } else if file_name == "scene.yml"
        || file_name == "scene.yaml"
        || file_name.ends_with(".scene.yml")
        || file_name.ends_with(".scene.yaml")
    {
        "sceneDocument"
    } else if file_name == "scene.rhai" || file_name.ends_with(".scene.rhai") {
        "sceneScript"
    } else if extension == "rhai" {
        "script"
    } else if file_name.ends_with(".font.yml") || file_name.ends_with(".font.yaml") {
        "font"
    } else if file_name.ends_with(".image.yml") || file_name.ends_with(".image.yaml") {
        "imageAsset"
    } else if file_name.ends_with(".tileset.yml")
        || file_name.ends_with(".tileset.yaml")
        || file_name.ends_with(".tile-ruleset.yml")
        || file_name.ends_with(".tile-ruleset.yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
        "tilemap"
    } else if file_name.ends_with(".sprite.yml")
        || file_name.ends_with(".sprite.yaml")
        || file_name.ends_with(".atlas.yml")
        || file_name.ends_with(".atlas.yaml")
    {
        "spritesheet"
    } else if file_name.ends_with(".particle.yml") || file_name.ends_with(".particle.yaml") {
        "particle"
    } else if file_name.ends_with(".audio.yml") || file_name.ends_with(".audio.yaml") {
        "audio"
    } else if file_name.ends_with(".material.yml") || file_name.ends_with(".material.yaml") {
        "material"
    } else if file_name.ends_with(".ui.yml") || file_name.ends_with(".ui.yaml") {
        "ui"
    } else if file_name.ends_with(".input.yml") || file_name.ends_with(".input.yaml") {
        "input"
    } else if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
        "rawImage"
    } else if matches!(extension.as_str(), "wav" | "ogg" | "mp3" | "flac") {
        "rawAudio"
    } else if matches!(extension.as_str(), "ttf" | "otf" | "woff" | "woff2") {
        "rawFont"
    } else if matches!(extension.as_str(), "yml" | "yaml") {
        "yaml"
    } else {
        "unknown"
    }
    .to_owned()
}

fn is_readable_project_text_kind(kind: &str) -> bool {
    matches!(
        kind,
        "manifest"
            | "sceneDocument"
            | "sceneScript"
            | "script"
            | "scriptPackage"
            | "tileset"
            | "tilemap"
            | "imageAsset"
            | "font"
            | "audio"
            | "particle"
            | "material"
            | "ui"
            | "input"
            | "yaml"
    )
}

fn language_for_project_file_kind(kind: &str) -> &'static str {
    match kind {
        "manifest" => "toml",
        "sceneDocument" | "scriptPackage" | "tileset" | "tilemap" | "imageAsset" | "font"
        | "audio" | "particle" | "material" | "ui" | "input" | "yaml" => "yaml",
        "sceneScript" | "script" => "rhai",
        _ => "text",
    }
}

fn asset_key_from_descriptor_relative_path(mod_id: &str, relative_path: &str) -> Option<String> {
    let normalized = relative_path.replace('\\', "/");
    let prefix = "assets/";
    let suffixes = [
        ".image.yml",
        ".sprite.yml",
        ".atlas.yml",
        ".tileset.yml",
        ".tile-ruleset.yml",
        ".tilemap.yml",
    ];
    let remainder = normalized.strip_prefix(prefix)?;
    let slash_index = remainder.find('/')?;
    let area = &remainder[..slash_index];
    let file_name = &remainder[slash_index + 1..];
    let suffix = suffixes.iter().find(|suffix| file_name.ends_with(**suffix))?;
    let asset_id = file_name.trim_end_matches(suffix).replace('\\', "/");
    Some(format!("{mod_id}/{area}/{asset_id}"))
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
