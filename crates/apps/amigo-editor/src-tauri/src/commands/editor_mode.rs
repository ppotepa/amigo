use std::fs;
use std::path::Path;

use amigo_scene::default_component_registry;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameSceneRequestDto {
    pub scene_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSceneComponentRequestDto {
    pub scene_id: String,
    pub component_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec2Dto {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSceneEntityRequestDto {
    pub scene_id: String,
    pub template_id: String,
    pub suggested_name: Option<String>,
    pub asset_key: Option<String>,
    pub position: Option<Vec2Dto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePatchResultDto {
    pub ok: bool,
    pub scene_dirty: bool,
    pub message: Option<String>,
}

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
    let scene_path = scene_document_path(Path::new(&session.root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    patch_scene_display_name(&mut document, &request.display_name)?;
    write_yaml_document(&scene_path, &document)?;

    let manifest_path = Path::new(&session.root_path).join("mod.toml");
    patch_manifest_scene_label(&manifest_path, &request.scene_id, &request.display_name)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!("Renamed scene to `{}`.", request.display_name)),
    })
}

pub fn add_scene_component(
    session_id: String,
    request: AddSceneComponentRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ScenePatchResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    let scene_path = scene_document_path(Path::new(&session.root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    let descriptor_default_yaml = default_component_registry()
        .descriptor_by_type_name(&request.component_type)
        .and_then(|descriptor| descriptor.default_yaml);
    let component_node =
        build_default_component_yaml(&request.component_type, descriptor_default_yaml)?;
    patch_add_scene_component(&mut document, component_node)?;
    write_yaml_document(&scene_path, &document)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!(
            "Added scene component `{}`.",
            request.component_type
        )),
    })
}

pub fn add_scene_entity(
    session_id: String,
    request: AddSceneEntityRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ScenePatchResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    let scene_path = scene_document_path(Path::new(&session.root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    let entity_node = build_entity_from_template(&document, &request);
    patch_add_scene_entity(&mut document, entity_node)?;
    write_yaml_document(&scene_path, &document)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!(
            "Added entity from template `{}`.",
            request.template_id
        )),
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

fn read_yaml_document(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    serde_yaml::from_str::<Value>(&text)
        .map_err(|error| format!("failed to parse `{}`: {error}", path.display()))
}

fn write_yaml_document(path: &Path, document: &Value) -> Result<(), String> {
    let text = serde_yaml::to_string(document)
        .map_err(|error| format!("failed to serialize `{}`: {error}", path.display()))?;
    fs::write(path, text).map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn patch_scene_display_name(document: &mut Value, display_name: &str) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let scene_key = Value::String("scene".to_owned());
    if let Some(scene_value) = root.get_mut(&scene_key) {
        let scene = scene_value
            .as_mapping_mut()
            .ok_or_else(|| "`scene` must be a mapping".to_owned())?;
        scene.insert(
            Value::String("name".to_owned()),
            Value::String(display_name.to_owned()),
        );
        scene.insert(
            Value::String("label".to_owned()),
            Value::String(display_name.to_owned()),
        );
    } else {
        root.insert(
            Value::String("name".to_owned()),
            Value::String(display_name.to_owned()),
        );
    }
    Ok(())
}

fn build_default_component_yaml(
    component_type: &str,
    default_yaml: Option<&str>,
) -> Result<Value, String> {
    let mut node = if let Some(default_yaml) = default_yaml {
        serde_yaml::from_str::<Value>(default_yaml).map_err(|error| {
            format!("failed to parse default YAML for `{component_type}`: {error}")
        })?
    } else {
        Value::Mapping(Mapping::new())
    };

    let mapping = node
        .as_mapping_mut()
        .ok_or_else(|| format!("default YAML for `{component_type}` must be a mapping"))?;
    mapping.insert(
        Value::String("type".to_owned()),
        Value::String(component_type.to_owned()),
    );
    Ok(node)
}

fn patch_add_scene_component(document: &mut Value, component_node: Value) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let components_key = Value::String("components".to_owned());
    if !root.contains_key(&components_key) {
        root.insert(components_key.clone(), Value::Sequence(Vec::new()));
    }

    let components = root
        .get_mut(&components_key)
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "`components` must be a sequence".to_owned())?;

    components.push(component_node);
    Ok(())
}

fn patch_add_scene_entity(document: &mut Value, entity: Value) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let entities_key = Value::String("entities".to_owned());
    if !root.contains_key(&entities_key) {
        root.insert(entities_key.clone(), Value::Sequence(Vec::new()));
    }

    let entities = root
        .get_mut(&entities_key)
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "`entities` must be a sequence".to_owned())?;
    entities.push(entity);
    Ok(())
}

fn build_entity_from_template(document: &Value, request: &AddSceneEntityRequestDto) -> Value {
    let entity_id = next_entity_id(document, &request.template_id);
    let entity_name = request
        .suggested_name
        .clone()
        .unwrap_or_else(|| entity_id.clone());

    let mut entity = Mapping::new();
    entity.insert(Value::String("id".to_owned()), Value::String(entity_id));
    entity.insert(Value::String("name".to_owned()), Value::String(entity_name));

    let mut components = Vec::new();
    match request.template_id.as_str() {
        "empty" => {}
        "sprite" => {
            components.push(component_with_transform(request.position.as_ref()));
            let mut sprite = Mapping::new();
            sprite.insert(
                Value::String("type".to_owned()),
                Value::String("Sprite2D".to_owned()),
            );
            if let Some(asset_key) = request
                .asset_key
                .as_ref()
                .filter(|value| !value.trim().is_empty())
            {
                sprite.insert(
                    Value::String("texture".to_owned()),
                    Value::String(asset_key.clone()),
                );
            }
            components.push(Value::Mapping(sprite));
        }
        "tilemap" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("TileMap2D"));
        }
        "trigger" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("Trigger2D"));
        }
        "camera" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("Camera2D"));
        }
        "spawnPoint" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("SpawnPoint"));
        }
        _ => {
            components.push(component_with_transform(request.position.as_ref()));
        }
    }

    entity.insert(
        Value::String("components".to_owned()),
        Value::Sequence(components),
    );
    Value::Mapping(entity)
}

fn component_with_transform(position: Option<&Vec2Dto>) -> Value {
    let mut transform = Mapping::new();
    transform.insert(
        Value::String("type".to_owned()),
        Value::String("Transform2D".to_owned()),
    );
    if let Some(position) = position {
        let mut translation = Mapping::new();
        translation.insert(
            Value::String("x".to_owned()),
            serde_yaml::to_value(position.x).unwrap_or(Value::from(0.0_f32)),
        );
        translation.insert(
            Value::String("y".to_owned()),
            serde_yaml::to_value(position.y).unwrap_or(Value::from(0.0_f32)),
        );
        transform.insert(
            Value::String("translation".to_owned()),
            Value::Mapping(translation),
        );
    }
    Value::Mapping(transform)
}

fn component_only_type(type_name: &str) -> Value {
    let mut component = Mapping::new();
    component.insert(
        Value::String("type".to_owned()),
        Value::String(type_name.to_owned()),
    );
    Value::Mapping(component)
}

fn next_entity_id(document: &Value, prefix: &str) -> String {
    let root = match document.as_mapping() {
        Some(root) => root,
        None => return format!("{prefix}_1"),
    };
    let entities = root
        .get(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mut next_index = 1_u32;
    loop {
        let candidate = format!("{prefix}_{next_index}");
        let exists = entities.iter().any(|entity| {
            entity
                .as_mapping()
                .and_then(|mapping| mapping.get(Value::String("id".to_owned())))
                .and_then(Value::as_str)
                .map(|id| id == candidate)
                .unwrap_or(false)
        });
        if !exists {
            return candidate;
        }
        next_index += 1;
    }
}

fn patch_manifest_scene_label(
    manifest_path: &Path,
    scene_id: &str,
    display_name: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;

    let mut in_scene = false;
    let mut current_scene_matches = false;
    let mut updated_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[scenes]]" {
            in_scene = true;
            current_scene_matches = false;
            updated_lines.push(line.to_owned());
            continue;
        }
        if in_scene && trimmed.starts_with("id = ") {
            current_scene_matches = trimmed.contains(&format!("\"{scene_id}\""));
            updated_lines.push(line.to_owned());
            continue;
        }
        if in_scene && current_scene_matches && trimmed.starts_with("label = ") {
            let leading = line
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>();
            updated_lines.push(format!(
                "{leading}label = \"{}\"",
                display_name.replace('"', "\\\"")
            ));
            continue;
        }
        if in_scene && trimmed.starts_with("[[") && trimmed != "[[scenes]]" {
            in_scene = false;
            current_scene_matches = false;
        }
        updated_lines.push(line.to_owned());
    }

    fs::write(manifest_path, format!("{}\n", updated_lines.join("\n")))
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))
}
