use super::super::document_snapshot::document_editor_snapshot_from_value;
use super::super::dto::{EditorCommandDto, EditorUiNodeSelectionDto};
use super::super::gizmos::enrich_snapshot_with_editor_control_state;
use super::super::session::EditorModeSession;
use super::super::transaction::{
    EditorTransaction, EditorTransactionFragment, new_transaction_id, now_ms,
};
use super::super::{
    patch_add_ui_node, patch_add_ui_template, patch_create_ui_document, patch_duplicate_ui_node,
    patch_move_ui_node, patch_remove_ui_node, patch_ui_node_property,
};

pub fn apply_session_command(
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
                Some(EditorUiNodeSelectionDto {
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
