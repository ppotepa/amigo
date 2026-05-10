use std::path::PathBuf;

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

use super::super::dto::{EditorCommandDto, EditorCommandResultDto};
use super::super::{
    apply_document_add_ui_node, apply_document_add_ui_template,
    apply_document_attached_local_offset_2d, apply_document_create_ui_document,
    apply_document_duplicate_ui_node, apply_document_move_ui_node, apply_document_remove_ui_node,
    apply_document_tilemap_marker_offset_2d, apply_document_transform_2d,
    apply_document_ui_node_property, document_editor_snapshot,
};

#[derive(Debug, Clone)]
pub struct LegacyEditorSession {
    pub mod_id: String,
    pub root_path: PathBuf,
}

pub fn apply_legacy_editor_command(
    session: LegacyEditorSession,
    command: EditorCommandDto,
) -> Result<EditorCommandResultDto, String> {
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

    match command {
        EditorCommandDto::SetEntityTransform2D {
            scene_id,
            entity_id,
            transform,
        } => map_document_command_result(
            apply_document_transform_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                transform,
            ),
            "Editor transform command failed.",
        ),
        EditorCommandDto::SetTileMapMarker2D {
            scene_id,
            entity_id,
            offset,
        } => map_document_command_result(
            apply_document_tilemap_marker_offset_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                offset,
            ),
            "Editor tilemap marker command failed.",
        ),
        EditorCommandDto::SetAttachedLocalOffset2D {
            scene_id,
            entity_id,
            local_offset,
        } => map_document_command_result(
            apply_document_attached_local_offset_2d(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                local_offset,
            ),
            "Editor attached offset command failed.",
        ),
        EditorCommandDto::SetUiNodeProperty {
            scene_id,
            entity_id,
            component_index,
            node_path,
            property_path,
            value,
        } => map_document_command_result(
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
        ),
        EditorCommandDto::CreateUiDocument {
            scene_id,
            entity_id,
            label,
            viewport_width,
            viewport_height,
            template,
        } => map_document_command_result(
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
        ),
        EditorCommandDto::AddUiNode {
            scene_id,
            entity_id,
            component_index,
            parent_path,
            node,
            insert_index,
        } => map_document_command_result(
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
        ),
        EditorCommandDto::AddUiTemplate {
            scene_id,
            entity_id,
            component_index,
            parent_path,
            template,
            id_prefix,
            insert_index,
        } => map_document_command_result(
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
        ),
        EditorCommandDto::DuplicateUiNode {
            scene_id,
            entity_id,
            component_index,
            node_path,
            new_id,
            copy_actions,
        } => map_document_command_result(
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
        ),
        EditorCommandDto::RemoveUiNode {
            scene_id,
            entity_id,
            component_index,
            node_path,
        } => map_document_command_result(
            apply_document_remove_ui_node(
                session.mod_id,
                session.root_path,
                scene_id,
                entity_id,
                component_index,
                node_path,
            ),
            "Editor remove UI node command failed.",
        ),
        EditorCommandDto::MoveUiNode {
            scene_id,
            entity_id,
            component_index,
            node_path,
            direction,
        } => map_document_command_result(
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
        ),
        unsupported => Ok(EditorCommandResultDto {
            ok: false,
            scene_dirty: false,
            changed_entities: changed_entities_for_command(&unsupported),
            snapshot: None,
            diagnostics: Vec::new(),
            message: Some(
                "Transform editing is disabled until editor-mode can patch real scene YAML/document data."
                    .to_owned(),
            ),
        }),
    }
}

fn changed_entities_for_command(command: &EditorCommandDto) -> Vec<String> {
    match command {
        EditorCommandDto::MoveEntity2D { entity_id, .. }
        | EditorCommandDto::SetEntityTransform2D { entity_id, .. }
        | EditorCommandDto::SetTileMapMarker2D { entity_id, .. }
        | EditorCommandDto::SetAttachedLocalOffset2D { entity_id, .. }
        | EditorCommandDto::SetUiNodeProperty { entity_id, .. }
        | EditorCommandDto::CreateUiDocument { entity_id, .. }
        | EditorCommandDto::AddUiNode { entity_id, .. }
        | EditorCommandDto::AddUiTemplate { entity_id, .. }
        | EditorCommandDto::DuplicateUiNode { entity_id, .. }
        | EditorCommandDto::RemoveUiNode { entity_id, .. }
        | EditorCommandDto::MoveUiNode { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SelectEntity { .. } => Vec::new(),
    }
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
            diagnostics: vec![EditorDiagnosticDto {
                level: DiagnosticLevel::Error,
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
