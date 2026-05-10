use std::fs;

use tauri::{AppHandle, State};

use crate::cache::root::EditorPaths;
use crate::editor_mode::document_snapshot::document_editor_snapshot_from_value;
use crate::editor_mode::dto::{
    EditorCommandDto, EditorCommandResultDto, EditorFrameResultDto, EditorModeDto,
    EditorPointerEventDto, EditorRenderTransportPreferenceDto, EditorSceneCanvasKindDto,
    EditorSceneSnapshotDto, EditorToolDto, EditorViewportDto, OpenEditorModeSessionResultDto,
};
use crate::editor_mode::gizmos::enrich_snapshot_with_editor_control_state;
pub use crate::editor_mode::scene_patches::{
    AddSceneComponentRequestDto, AddSceneEntityRequestDto, RenameSceneRequestDto,
    ScenePatchResultDto,
};
use crate::editor_mode::transaction::{
    apply_transaction_after, apply_transaction_after_to_document, apply_transaction_before,
    apply_transaction_before_to_document,
};
use crate::editor_mode::{
    EditorModeSession, EditorModeSessionRegistry, EditorTransactionLog, default_editor_cursor,
    discard_editor_mode_session_changes as discard_editor_mode_session_changes_impl,
    document_editor_snapshot, enrich_snapshot_with_editor_state, fallback_editor_snapshot,
    handle_editor_pointer_event, new_editor_mode_session_id, render_editor_mode_frame,
    resolve_transport_kind, save_editor_mode_session_changes, scene_document_path,
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

pub fn rename_scene(
    session_id: String,
    request: RenameSceneRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ScenePatchResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::editor_mode::scene_patches::rename_scene(&session.root_path, request)
}

pub fn add_scene_component(
    session_id: String,
    request: AddSceneComponentRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ScenePatchResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::editor_mode::scene_patches::add_scene_component(&session.root_path, request)
}

pub fn add_scene_entity(
    session_id: String,
    request: AddSceneEntityRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ScenePatchResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::editor_mode::scene_patches::add_scene_entity(&session.root_path, request)
}

pub fn apply_editor_command(
    session_id: String,
    command: EditorCommandDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<EditorCommandResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::editor_mode::commands::apply_legacy_editor_command(
        crate::editor_mode::commands::LegacyEditorSession {
            mod_id: session.mod_id,
            root_path: session.root_path.into(),
        },
        command,
    )
}

pub async fn apply_editor_mode_command(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    command: EditorCommandDto,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let mut message = "Editor mode command applied to session.".to_owned();
    let updated = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        message = crate::editor_mode::commands::apply_session_command(session, command)?;
        Ok(())
    })?;
    let frame = render_editor_mode_frame(app, &paths, &updated).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(updated.dto()),
        snapshot: Some(updated.snapshot.clone()),
        frame: Some(frame),
        diagnostics: updated.diagnostics.clone(),
        message: Some(message),
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
    let scene_path = scene_document_path(std::path::Path::new(&session.root_path), &scene_id);
    let document_text = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "failed to read scene document `{}`: {error}",
            scene_path.display()
        )
    })?;
    let document_value =
        serde_yaml::from_str::<serde_yaml::Value>(&document_text).map_err(|error| {
            format!(
                "failed to parse scene document `{}`: {error}",
                scene_path.display()
            )
        })?;
    let mut snapshot =
        document_editor_snapshot(session.mod_id.clone(), &session.root_path, scene_id.clone())?;
    snapshot.canvas_kind = refine_canvas_kind(snapshot.canvas_kind, &scene_id);
    let snapshot = enrich_snapshot_with_editor_state(snapshot, None, EditorToolDto::Select);

    let transport = resolve_transport_kind(transport_preference);
    let editor_mode_session_id = new_editor_mode_session_id(&session_id, &scene_id);

    let editor_session = EditorModeSession {
        editor_mode_session_id,
        editor_session_id: session_id,
        mod_id: session.mod_id,
        root_path: session.root_path.into(),
        scene_id,
        document_value,
        mode: EditorModeDto::Edit,
        tool: EditorToolDto::Select,
        viewport,
        transport,
        dirty: false,
        revision: 1,
        saved_revision: 1,
        selected_entity_id: None,
        selected_ui_node: None,
        active_interaction: None,
        hovered_control_id: None,
        hovered_handle_id: None,
        hovered_entity_id: None,
        hovered_ui_node: None,
        active_control_id: None,
        active_handle_id: None,
        cursor: default_editor_cursor(),
        last_pointer_scene_x: None,
        last_pointer_scene_y: None,
        last_pointer_frame_x: None,
        last_pointer_frame_y: None,
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

pub fn get_editor_mode_scene_hierarchy(
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<crate::dto::EditorSceneHierarchyDto, String> {
    let session = editor_mode_sessions.get(&editor_mode_session_id)?;
    project_tree::scene_hierarchy_from_value(
        session.mod_id,
        session.scene_id,
        &session.document_value,
    )
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
        && optional_f32_eq(a.camera_x, b.camera_x)
        && optional_f32_eq(a.camera_y, b.camera_y)
        && optional_f32_eq(a.zoom, b.zoom)
}

fn optional_f32_eq(a: Option<f32>, b: Option<f32>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => (a - b).abs() < 0.001,
        (None, None) => true,
        _ => false,
    }
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
        session.snapshot = enrich_snapshot_with_editor_state(
            session.snapshot.clone(),
            session.selected_entity_id.clone(),
            session.tool,
        );
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
    discard_editor_mode_session_changes_impl(
        app,
        &paths,
        &editor_mode_sessions,
        editor_mode_session_id,
    )
    .await
}

pub async fn undo_editor_mode_transaction(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let mut message = "Nothing to undo.".to_owned();
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        if let Some(transaction) = session.transactions.undo() {
            let selected_ui_node =
                apply_transaction_before_to_document(&mut session.document_value, &transaction);
            if let Some(selected_ui_node) = selected_ui_node {
                session.selected_ui_node = selected_ui_node;
                session.selected_entity_id = session
                    .selected_ui_node
                    .as_ref()
                    .map(|selection| selection.entity_id.clone());
                let snapshot = document_editor_snapshot_from_value(
                    session.mod_id.clone(),
                    session.scene_id.clone(),
                    &session.document_value,
                )?;
                session.snapshot = enrich_snapshot_with_editor_control_state(
                    snapshot,
                    session.selected_entity_id.clone(),
                    session.selected_ui_node.clone(),
                    session.tool,
                    Default::default(),
                );
            } else {
                apply_transaction_before(&mut session.snapshot, &transaction);
                session.selected_entity_id = transaction.changed_entities.first().cloned();
                session.snapshot = enrich_snapshot_with_editor_state(
                    session.snapshot.clone(),
                    session.selected_entity_id.clone(),
                    session.tool,
                );
            }
            session.dirty = session.transactions.is_dirty();
            message = format!("Undid {}.", transaction.label);
        }
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
        message: Some(message),
    })
}

pub async fn redo_editor_mode_transaction(
    app: AppHandle,
    paths: State<'_, EditorPaths>,
    _session_id: String,
    editor_mode_session_id: String,
    editor_mode_sessions: State<'_, EditorModeSessionRegistry>,
) -> Result<EditorFrameResultDto, String> {
    let mut message = "Nothing to redo.".to_owned();
    let session = editor_mode_sessions.update(&editor_mode_session_id, |session| {
        if let Some(transaction) = session.transactions.redo() {
            let selected_ui_node =
                apply_transaction_after_to_document(&mut session.document_value, &transaction);
            if let Some(selected_ui_node) = selected_ui_node {
                session.selected_ui_node = selected_ui_node;
                session.selected_entity_id = session
                    .selected_ui_node
                    .as_ref()
                    .map(|selection| selection.entity_id.clone());
                let snapshot = document_editor_snapshot_from_value(
                    session.mod_id.clone(),
                    session.scene_id.clone(),
                    &session.document_value,
                )?;
                session.snapshot = enrich_snapshot_with_editor_control_state(
                    snapshot,
                    session.selected_entity_id.clone(),
                    session.selected_ui_node.clone(),
                    session.tool,
                    Default::default(),
                );
            } else {
                apply_transaction_after(&mut session.snapshot, &transaction);
                session.selected_entity_id = transaction.changed_entities.first().cloned();
                session.snapshot = enrich_snapshot_with_editor_state(
                    session.snapshot.clone(),
                    session.selected_entity_id.clone(),
                    session.tool,
                );
            }
            session.dirty = session.transactions.is_dirty();
            message = format!("Redid {}.", transaction.label);
        }
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
        message: Some(message),
    })
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
