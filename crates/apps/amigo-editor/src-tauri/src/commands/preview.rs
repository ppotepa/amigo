use tauri::{AppHandle, State};

use crate::cache as app_cache;
use crate::cache::root::EditorPaths;
use crate::dto::{ScenePreviewDto, ScenePreviewFrameGeneratedDto};
use crate::events::bus;
use crate::events::preview_progress::{PreviewProgressPayload, emit_preview_progress};
use crate::mods::discovery::discover_editor_mods;
use crate::preview::renderer;

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
            app_cache::project_id::project_cache_id_for_root(&discovered_mod.root_path);
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
