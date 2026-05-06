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
use crate::editor_mode::transaction::{
    EditorTransaction, EditorTransactionFragment, apply_transaction_after,
    apply_transaction_after_to_document, apply_transaction_before,
    apply_transaction_before_to_document, new_transaction_id, now_ms,
};
use crate::editor_mode::{
    EditorModeSession, EditorModeSessionRegistry, EditorTransactionLog, apply_document_add_ui_node,
    apply_document_add_ui_template, apply_document_attached_local_offset_2d,
    apply_document_create_ui_document, apply_document_duplicate_ui_node,
    apply_document_move_ui_node, apply_document_remove_ui_node,
    apply_document_tilemap_marker_offset_2d, apply_document_transform_2d,
    apply_document_ui_node_property, default_editor_cursor,
    discard_editor_mode_session_changes as discard_editor_mode_session_changes_impl,
    document_editor_snapshot, enrich_snapshot_with_editor_state, fallback_editor_snapshot,
    handle_editor_pointer_event, new_editor_mode_session_id, patch_add_ui_node,
    patch_add_ui_template, patch_create_ui_document, patch_duplicate_ui_node, patch_move_ui_node,
    patch_remove_ui_node, patch_ui_node_property, render_editor_mode_frame, resolve_transport_kind,
    save_editor_mode_session_changes, scene_document_path,
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

    if let EditorCommandDto::MoveEntity2D {
        scene_id,
        entity_id,
        dx,
        dy,
    } = &command
    {
        let snapshot =
            document_editor_snapshot(session.mod_id.clone(), &session.root_path, scene_id.clone())?;
        let Some(object) = snapshot
            .objects
            .iter()
            .find(|object| object.entity_id == *entity_id)
        else {
            return Ok(EditorCommandResultDto {
                ok: false,
                scene_dirty: false,
                changed_entities: Vec::new(),
                snapshot: None,
                diagnostics: Vec::new(),
                message: Some(format!(
                    "Entity `{entity_id}` was not found in editor snapshot."
                )),
            });
        };
        let Some(mut transform) = object.transform_2.clone() else {
            return Ok(EditorCommandResultDto {
                ok: false,
                scene_dirty: false,
                changed_entities: Vec::new(),
                snapshot: None,
                diagnostics: Vec::new(),
                message: Some(format!("Entity `{entity_id}` has no editable transform2.")),
            });
        };
        transform.x += *dx;
        transform.y += *dy;
        return map_document_command_result(
            apply_document_transform_2d(
                session.mod_id,
                session.root_path,
                scene_id.clone(),
                entity_id.clone(),
                transform,
            ),
            "Editor move command failed.",
        );
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

    if let EditorCommandDto::SetUiNodeProperty {
        scene_id,
        entity_id,
        component_index,
        node_path,
        property_path,
        value,
    } = command
    {
        return map_document_command_result(
            apply_document_ui_node_property(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                node_path,
                property_path,
                value,
            ),
            "Editor UI node command failed.",
        );
    }

    if let EditorCommandDto::CreateUiDocument {
        scene_id,
        entity_id,
        label,
        viewport_width,
        viewport_height,
        template,
    } = command
    {
        return map_document_command_result(
            apply_document_create_ui_document(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                label,
                viewport_width,
                viewport_height,
                template,
            ),
            "Editor create UI document command failed.",
        );
    }

    if let EditorCommandDto::AddUiNode {
        scene_id,
        entity_id,
        component_index,
        parent_path,
        node,
        insert_index,
    } = command
    {
        return map_document_command_result(
            apply_document_add_ui_node(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                parent_path,
                node,
                insert_index,
            ),
            "Editor add UI node command failed.",
        );
    }

    if let EditorCommandDto::AddUiTemplate {
        scene_id,
        entity_id,
        component_index,
        parent_path,
        template,
        id_prefix,
        insert_index,
    } = command
    {
        return map_document_command_result(
            apply_document_add_ui_template(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                parent_path,
                template,
                id_prefix,
                insert_index,
            ),
            "Editor add UI template command failed.",
        );
    }

    if let EditorCommandDto::DuplicateUiNode {
        scene_id,
        entity_id,
        component_index,
        node_path,
        new_id,
        copy_actions,
    } = command
    {
        return map_document_command_result(
            apply_document_duplicate_ui_node(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                node_path,
                new_id,
                copy_actions,
            ),
            "Editor duplicate UI node command failed.",
        );
    }

    if let EditorCommandDto::RemoveUiNode {
        scene_id,
        entity_id,
        component_index,
        node_path,
    } = command
    {
        return map_document_command_result(
            apply_document_remove_ui_node(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                node_path,
            ),
            "Editor remove UI node command failed.",
        );
    }

    if let EditorCommandDto::MoveUiNode {
        scene_id,
        entity_id,
        component_index,
        node_path,
        direction,
    } = command
    {
        return map_document_command_result(
            apply_document_move_ui_node(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                node_path,
                direction,
            ),
            "Editor move UI node command failed.",
        );
    }

    let changed_entities = match &command {
        EditorCommandDto::MoveEntity2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetEntityTransform2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetTileMapMarker2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetAttachedLocalOffset2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetUiNodeProperty { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::CreateUiDocument { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::AddUiNode { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::AddUiTemplate { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::DuplicateUiNode { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::RemoveUiNode { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::MoveUiNode { entity_id, .. } => vec![entity_id.clone()],
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
        message = apply_editor_mode_command_to_session(session, command)?;
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

fn apply_editor_mode_command_to_session(
    session: &mut EditorModeSession,
    command: EditorCommandDto,
) -> Result<String, String> {
    let before = session.document_value.clone();
    let selected_before = session.selected_ui_node.clone();
    let mut document = before.clone();

    let (label, changed_entity_id, selected_after, message) = match command {
        EditorCommandDto::SetUiNodeProperty {
            entity_id,
            component_index,
            node_path,
            property_path,
            value,
            ..
        } => {
            patch_ui_node_property(
                &mut document,
                &entity_id,
                component_index,
                &node_path,
                &property_path,
                value,
            )?;
            (
                format!("Set UI node property {property_path}"),
                entity_id.clone(),
                Some(crate::editor_mode::dto::EditorUiNodeSelectionDto {
                    entity_id,
                    component_index,
                    node_path,
                }),
                format!("UI node property `{property_path}` was updated."),
            )
        }
        EditorCommandDto::CreateUiDocument {
            entity_id,
            label,
            viewport_width,
            viewport_height,
            template,
            ..
        } => {
            let outcome = patch_create_ui_document(
                &mut document,
                &entity_id,
                &label,
                viewport_width,
                viewport_height,
                template,
            )?;
            (
                "Create UI document".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        EditorCommandDto::AddUiNode {
            entity_id,
            component_index,
            parent_path,
            node,
            insert_index,
            ..
        } => {
            let outcome = patch_add_ui_node(
                &mut document,
                &entity_id,
                component_index,
                &parent_path,
                node,
                insert_index,
            )?;
            (
                "Add UI node".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        EditorCommandDto::AddUiTemplate {
            entity_id,
            component_index,
            parent_path,
            template,
            id_prefix,
            insert_index,
            ..
        } => {
            let outcome = patch_add_ui_template(
                &mut document,
                &entity_id,
                component_index,
                &parent_path,
                template,
                &id_prefix,
                insert_index,
            )?;
            (
                "Add UI template".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        EditorCommandDto::DuplicateUiNode {
            entity_id,
            component_index,
            node_path,
            new_id,
            copy_actions,
            ..
        } => {
            let outcome = patch_duplicate_ui_node(
                &mut document,
                &entity_id,
                component_index,
                &node_path,
                new_id,
                copy_actions,
            )?;
            (
                "Duplicate UI node".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        EditorCommandDto::RemoveUiNode {
            entity_id,
            component_index,
            node_path,
            ..
        } => {
            let outcome =
                patch_remove_ui_node(&mut document, &entity_id, component_index, &node_path)?;
            (
                "Remove UI node".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        EditorCommandDto::MoveUiNode {
            entity_id,
            component_index,
            node_path,
            direction,
            ..
        } => {
            let outcome = patch_move_ui_node(
                &mut document,
                &entity_id,
                component_index,
                &node_path,
                direction,
            )?;
            (
                "Move UI node".to_owned(),
                outcome.changed_entity_id,
                outcome.selected_ui_node,
                outcome.message,
            )
        }
        _ => return Err("command is not supported by apply_editor_mode_command yet".to_owned()),
    };

    let after = document.clone();
    let snapshot = document_editor_snapshot_from_value(
        session.mod_id.clone(),
        session.scene_id.clone(),
        &document,
    )?;
    session.document_value = document;
    session.selected_entity_id = Some(changed_entity_id.clone());
    session.selected_ui_node = selected_after.clone();
    session.snapshot = enrich_snapshot_with_editor_control_state(
        snapshot,
        session.selected_entity_id.clone(),
        session.selected_ui_node.clone(),
        session.tool,
        Default::default(),
    );
    let next_revision = session.revision + 1;
    session.transactions.push(EditorTransaction {
        id: new_transaction_id(next_revision, &label.to_lowercase().replace(' ', "-")),
        label,
        kind: "ui-document-value".to_owned(),
        target: selected_after
            .as_ref()
            .map(|selection| {
                format!(
                    "{}:{}:{}",
                    selection.entity_id, selection.component_index, selection.node_path
                )
            })
            .unwrap_or_else(|| changed_entity_id.clone()),
        revision: next_revision,
        timestamp_ms: now_ms(),
        changed_entities: vec![changed_entity_id],
        fragments: vec![EditorTransactionFragment::UiDocumentValue {
            entity_id: session
                .selected_entity_id
                .clone()
                .unwrap_or_else(|| session.scene_id.clone()),
            before,
            after,
            selected_before,
            selected_after,
        }],
    });
    session.dirty = true;
    session.bump_revision();
    Ok(message)
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
