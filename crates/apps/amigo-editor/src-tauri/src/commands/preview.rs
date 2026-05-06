use tauri::{AppHandle, State};

use crate::cache::root::EditorPaths;
use crate::dto::ScenePreviewDto;
use crate::preview;

pub async fn request_scene_preview(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    mod_id: String,
    scene_id: String,
    force_regenerate: Option<bool>,
) -> Result<ScenePreviewDto, String> {
    preview::request_scene_preview_dto(
        app,
        &paths,
        mod_id,
        scene_id,
        force_regenerate.unwrap_or(false),
    )
    .await
}
