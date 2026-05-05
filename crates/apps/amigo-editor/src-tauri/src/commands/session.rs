use tauri::{AppHandle, State};

use crate::dto::{EditorSessionDto, OpenModResultDto};
use crate::events::bus;
use crate::mods::discovery::discover_editor_mods;
use crate::preview::renderer;
use crate::settings::editor_settings::{load_editor_settings, save_editor_settings};
use crate::windows::descriptors::EditorWindowKind;
use crate::windows::manager::open_or_focus_window;
use crate::{cache::root::EditorPaths, session::EditorSessionRegistry};

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

pub fn get_editor_session(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorSessionDto, String> {
    sessions.get_session(&session_id)
}

pub fn close_editor_session(
    app: AppHandle,
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<(), String> {
    sessions.close_session(&session_id)?;
    bus::emit_session_closed(&app, session_id)
}

pub async fn regenerate_all_scene_previews(
    paths: State<'_, EditorPaths>,
    mod_id: String,
) -> Result<Vec<crate::dto::ScenePreviewDto>, String> {
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
