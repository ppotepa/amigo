use tauri::AppHandle;

use crate::cache::root::EditorPaths;
use crate::preview;

use super::coordinates::{EditorCamera2D, EditorCoordinateMapper, EditorFrameSize};
use super::dto::EditorFrameDto;
use super::overlay::compose_editor_overlay_image_url;
use super::render_transport::{EditorRenderedFrame, publish_editor_frame};
use super::session::EditorModeSession;

pub async fn render_editor_mode_frame(
    app: AppHandle,
    paths: &EditorPaths,
    session: &EditorModeSession,
) -> Result<EditorFrameDto, String> {
    let preview = preview::request_scene_preview_dto(
        app,
        paths,
        session.mod_id.clone(),
        session.scene_id.clone(),
        false,
    )
    .await?;

    let image_url = preview
        .image_url
        .or_else(|| preview.frame_urls.first().cloned());
    let width = preview.width.max(1);
    let height = preview.height.max(1);
    let mapper = EditorCoordinateMapper {
        frame: EditorFrameSize {
            width: width as f32,
            height: height as f32,
        },
        camera: EditorCamera2D {
            center_x: session.snapshot.camera.x,
            center_y: session.snapshot.camera.y,
            zoom: session.snapshot.camera.zoom,
        },
    };
    let image_url = compose_editor_overlay_image_url(
        image_url,
        width,
        height,
        &session.snapshot,
        session,
        mapper,
    );

    let rendered = EditorRenderedFrame {
        session_id: session.editor_mode_session_id.clone(),
        revision: session.revision,
        width,
        height,
        device_pixel_ratio: session.viewport.device_pixel_ratio,
        image_url,
        rgba: Vec::new(),
    };

    publish_editor_frame(session.transport, rendered)
}
