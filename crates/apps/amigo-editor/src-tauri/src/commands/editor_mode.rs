use tauri::{AppHandle, State};

use crate::cache::root::EditorPaths;
use crate::editor_mode::dto::{
    EditorCameraDto, EditorCommandDto, EditorCommandResultDto, EditorFrameResultDto,
    EditorHitTestCandidateDto, EditorHitTestResultDto, EditorModeDto, EditorPointerEventDto,
    EditorRenderTransportPreferenceDto, EditorSceneCanvasKindDto, EditorSceneSnapshotDto,
    EditorToolDto, EditorViewportDto, EditorViewportPointDto, OpenEditorModeSessionResultDto,
};
use crate::editor_mode::{
    EditorModeSession, EditorModeSessionRegistry, EditorTransactionLog,
    apply_document_attached_local_offset_2d, apply_document_tilemap_marker_offset_2d,
    apply_document_transform_2d, document_editor_snapshot, fallback_editor_snapshot,
    handle_editor_pointer_event, new_editor_mode_session_id, render_editor_mode_frame,
    resolve_transport_kind, save_editor_mode_session_changes,
};
use crate::session::EditorSessionRegistry;

use super::project_tree;

pub fn get_editor_scene_snapshot(
    session_id: String,
    scene_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorSceneSnapshotDto, String> {
    let session = sessions.get_session(&session_id)?;
    let entity_count = project_tree::get_scene_hierarchy(session.mod_id.clone(), scene_id.clone())
        .map(|hierarchy| hierarchy.entities.len())
        .unwrap_or(0);
    match document_editor_snapshot(session.mod_id.clone(), &session.root_path, scene_id.clone()) {
        Ok(mut snapshot) => {
            snapshot.canvas_kind = refine_canvas_kind(snapshot.canvas_kind, &scene_id);
            Ok(snapshot)
        }
        Err(error) => {
            let mut snapshot =
                fallback_editor_snapshot(session.mod_id, scene_id.clone(), entity_count);
            snapshot.canvas_kind = infer_scene_canvas_kind(&scene_id);
            snapshot.diagnostics.push(crate::dto::EditorDiagnosticDto {
                level: crate::dto::DiagnosticLevel::Info,
                code: "EDITOR_MODE_DOCUMENT_SNAPSHOT_FAILED".to_owned(),
                message: error,
                path: None,
            });
            Ok(snapshot)
        }
    }
}

pub fn hit_test_editor_scene(
    session_id: String,
    scene_id: String,
    point: EditorViewportPointDto,
    camera: EditorCameraDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorHitTestResultDto, String> {
    let snapshot = get_editor_scene_snapshot(session_id, scene_id, sessions)?;
    let _ = camera;
    let mut candidates = Vec::new();

    for object in snapshot.objects.iter().rev() {
        let Some(bounds) = &object.bounds_2 else {
            continue;
        };
        if point.x >= bounds.x
            && point.x <= bounds.x + bounds.width
            && point.y >= bounds.y
            && point.y <= bounds.y + bounds.height
        {
            candidates.push(EditorHitTestCandidateDto {
                entity_id: object.entity_id.clone(),
                name: object.name.clone(),
                depth: candidates.len() as i32,
                bounds_2: object.bounds_2.clone(),
            });
        }
    }

    Ok(EditorHitTestResultDto {
        hit: !candidates.is_empty(),
        entity_id: candidates
            .first()
            .map(|candidate| candidate.entity_id.clone()),
        candidates,
    })
}

pub fn apply_editor_command(
    session_id: String,
    command: EditorCommandDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorCommandResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    if let EditorCommandDto::SelectEntity { entity_id, .. } = &command {
        return Ok(EditorCommandResultDto {
            ok: true,
            scene_dirty: false,
            changed_entities: vec![entity_id.clone()],
            snapshot: None,
            diagnostics: Vec::new(),
            message: Some("Entity selected in editor context.".to_owned()),
        });
    }

    if let EditorCommandDto::SetEntityTransform2D {
        scene_id,
        entity_id,
        transform,
    } = command
    {
        return map_document_command_result(
            apply_document_transform_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                transform,
            ),
            "Editor transform command failed.",
        );
    }

    if let EditorCommandDto::SetTileMapMarker2D {
        scene_id,
        entity_id,
        offset,
    } = command
    {
        return map_document_command_result(
            apply_document_tilemap_marker_offset_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                offset,
            ),
            "Editor tilemap marker command failed.",
        );
    }

    if let EditorCommandDto::SetAttachedLocalOffset2D {
        scene_id,
        entity_id,
        local_offset,
    } = command
    {
        return map_document_command_result(
            apply_document_attached_local_offset_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                local_offset,
            ),
            "Editor attached offset command failed.",
        );
    }

    let changed_entities = match &command {
        EditorCommandDto::MoveEntity2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetEntityTransform2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetTileMapMarker2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetAttachedLocalOffset2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SelectEntity { .. } => Vec::new(),
    };

    Ok(EditorCommandResultDto {
        ok: false,
        scene_dirty: false,
        changed_entities,
        snapshot: None,
        diagnostics: Vec::new(),
        message: Some(
            "Transform editing is disabled until editor-mode can patch real scene YAML/document data."
                .to_owned(),
        ),
    })
}

pub async fn open_editor_mode_session(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    session_id: String,
    scene_id: String,
    viewport: EditorViewportDto,
    transport_preference: EditorRenderTransportPreferenceDto,
    sessions: State<'_, EditorSessionRegistry>,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<OpenEditorModeSessionResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    let mut snapshot =
        document_editor_snapshot(session.mod_id.clone(), &session.root_path, scene_id.clone())?;
    snapshot.canvas_kind = refine_canvas_kind(snapshot.canvas_kind, &scene_id);

    let transport = resolve_transport_kind(transport_preference);
    let editor_mode_session_id = new_editor_mode_session_id(&session_id, &scene_id);

    let editor_session = EditorModeSession {
        editor_mode_session_id,
        editor_session_id: session_id,
        mod_id: session.mod_id,
        root_path: session.root_path.into(),
        scene_id,
        mode: EditorModeDto::Edit,
        tool: EditorToolDto::Select,
        viewport,
        transport,
        dirty: false,
        revision: 1,
        selected_entity_id: None,
        transactions: EditorTransactionLog::default(),
        snapshot,
        diagnostics: Vec::new(),
    };

    let frame = render_editor_mode_frame(app, &paths, &editor_session).await?;
    let session_dto = editor_mode_sessions.insert(editor_session.clone())?;

    Ok(OpenEditorModeSessionResultDto {
        session: session_dto,
        snapshot: editor_session.snapshot,
        frame,
        diagnostics: editor_session.diagnostics,
    })
}

pub fn close_editor_mode_session(
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<(), String> {
    editor_mode_sessions.remove(&editor_mode_session_id)
}

pub async fn get_editor_mode_frame(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let session = editor_mode_sessions.get(&editor_mode_session_id)?;
    let frame = render_editor_mode_frame(app, &paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}

pub async fn resize_editor_mode_viewport(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    viewport: EditorViewportDto,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        if !same_editor_viewport(&session.viewport, &viewport) {
            session.viewport = viewport;
            session.bump_revision();
        }
        Ok(())
    })?;
    let frame = render_editor_mode_frame(app, &paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}

fn same_editor_viewport(a: &EditorViewportDto, b: &EditorViewportDto) -> bool {
    (a.css_width - b.css_width).abs() < 0.5
        && (a.css_height - b.css_height).abs() < 0.5
        && a.render_width == b.render_width
        && a.render_height == b.render_height
        && (a.device_pixel_ratio - b.device_pixel_ratio).abs() < 0.001
}

pub async fn send_editor_pointer_event(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    event: EditorPointerEventDto,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    handle_editor_pointer_event(
        app,
        &paths,
        &editor_mode_sessions,
        editor_mode_session_id,
        event,
    )
    .await
}

pub async fn set_editor_mode(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    mode: EditorModeDto,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        session.mode = mode;
        session.bump_revision();
        Ok(())
    })?;
    let frame = render_editor_mode_frame(app, &paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}

pub async fn set_editor_tool(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    tool: EditorToolDto,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        session.tool = tool;
        session.bump_revision();
        Ok(())
    })?;
    let frame = render_editor_mode_frame(app, &paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}

pub async fn save_editor_mode_session(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    save_editor_mode_session_changes(app, &paths, &editor_mode_sessions, editor_mode_session_id)
        .await
}

pub async fn discard_editor_mode_session_changes(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        session.dirty = false;
        session.bump_revision();
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, &paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor mode changes discarded.".to_owned()),
    })
}

fn map_document_command_result(
    result: Result<EditorCommandResultDto, String>,
    message: &str,
) -> Result<EditorCommandResultDto, String> {
    match result {
        Ok(result) => Ok(result),
        Err(error) => Ok(EditorCommandResultDto {
            ok: false,
            scene_dirty: false,
            changed_entities: Vec::new(),
            snapshot: None,
            diagnostics: vec![crate::dto::EditorDiagnosticDto {
                level: crate::dto::DiagnosticLevel::Error,
                code: error
                    .split(':')
                    .next()
                    .unwrap_or("EDITOR_COMMAND_FAILED")
                    .to_owned(),
                message: error,
                path: None,
            }],
            message: Some(message.to_owned()),
        }),
    }
}

fn refine_canvas_kind(
    current: EditorSceneCanvasKindDto,
    scene_id: &str,
) -> EditorSceneCanvasKindDto {
    let inferred = infer_scene_canvas_kind(scene_id);
    if matches!(
        inferred,
        EditorSceneCanvasKindDto::ThreeD | EditorSceneCanvasKindDto::TwoHalfD
    ) {
        return inferred;
    }
    current
}

fn infer_scene_canvas_kind(scene_id: &str) -> EditorSceneCanvasKindDto {
    let normalized = scene_id.to_lowercase();
    if normalized.contains("2.5d")
        || normalized.contains("2-5d")
        || normalized.contains("isometric")
        || normalized.contains("iso")
    {
        return EditorSceneCanvasKindDto::TwoHalfD;
    }

    if normalized.contains("3d")
        || normalized.contains("mesh")
        || normalized.contains("material")
        || normalized.contains("cube")
    {
        return EditorSceneCanvasKindDto::ThreeD;
    }

    EditorSceneCanvasKindDto::TwoD
}
