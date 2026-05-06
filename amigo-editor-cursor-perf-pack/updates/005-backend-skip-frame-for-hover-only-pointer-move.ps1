param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$file = Get-AmigoFile $RepoRoot "crates/apps/amigo-editor/src-tauri/src/editor_mode/input.rs"
$text = Read-AmigoText $file

if ($text -match "should_render_editor_pointer_frame") {
  Write-Host "SKIP input.rs already contains should_render_editor_pointer_frame"
  return
}

$old = @'
pub async fn handle_editor_pointer_event(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
    event: EditorPointerEventDto,
) -> Result<EditorFrameResultDto, String> {
    let session = registry.update(&editor_mode_session_id, |session| {
        session.viewport = event.viewport.clone();
        session.last_pointer_scene_x = Some(event.scene_x());
        session.last_pointer_scene_y = Some(event.scene_y());
        session.last_pointer_frame_x = event.frame_x();
        session.last_pointer_frame_y = event.frame_y();

        match event.r#type.as_str() {
            "pointerDown" => handle_pointer_down(session, &event),
            "pointerMove" => handle_pointer_move(session, &event),
            "pointerUp" => handle_pointer_up(session),
            "pointerCancel" => handle_pointer_cancel(session),
            "wheel" => session.bump_revision(),
            _ => {}
        }

        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}
'@

$new = @'
pub async fn handle_editor_pointer_event(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
    event: EditorPointerEventDto,
) -> Result<EditorFrameResultDto, String> {
    let event_type = event.r#type.clone();
    let event_buttons = event.buttons.unwrap_or_default();

    let session = registry.update(&editor_mode_session_id, |session| {
        session.viewport = event.viewport.clone();
        session.last_pointer_scene_x = Some(event.scene_x());
        session.last_pointer_scene_y = Some(event.scene_y());
        session.last_pointer_frame_x = event.frame_x();
        session.last_pointer_frame_y = event.frame_y();

        match event.r#type.as_str() {
            "pointerDown" => handle_pointer_down(session, &event),
            "pointerMove" => handle_pointer_move(session, &event),
            "pointerUp" => handle_pointer_up(session),
            "pointerCancel" => handle_pointer_cancel(session),
            "wheel" => session.bump_revision(),
            _ => {}
        }

        Ok(())
    })?;

    let frame = if should_render_editor_pointer_frame(&session, &event_type, event_buttons) {
        Some(render_editor_mode_frame(app, paths, &session).await?)
    } else {
        None
    };

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame,
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}

fn should_render_editor_pointer_frame(
    session: &EditorModeSession,
    event_type: &str,
    event_buttons: u16,
) -> bool {
    match event_type {
        // Hover-only mouse movement should not re-render the image-url frame.
        // It is handled by the local lightweight cursor overlay in the React viewport.
        "pointerMove" => session.active_interaction.is_some() || event_buttons != 0,
        _ => true,
    }
}
'@

if (!$text.Contains($old)) {
  throw "Could not find exact handle_editor_pointer_event block. Update input.rs manually or regenerate this pack against current code."
}

$text = $text.Replace($old, $new)
Write-AmigoText $file $text
Write-Host "OK updated input.rs hover-only render policy"
