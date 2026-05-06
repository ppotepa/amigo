use tauri::AppHandle;

use crate::cache::root::EditorPaths;
use crate::editor_mode::document_commands::{patch_entity_transform_2d, patch_prefab_override};
use crate::editor_mode::document_snapshot::{
    document_editor_snapshot, document_editor_snapshot_from_value,
};

use super::dto::EditorFrameResultDto;
use super::gizmos::enrich_snapshot_with_editor_control_state;
use super::renderer::render_editor_mode_frame;
use super::session::{EditorModeSession, EditorModeSessionRegistry};
use super::transaction::EditorTransactionFragment;

pub async fn save_editor_mode_session_changes(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
) -> Result<EditorFrameResultDto, String> {
    let session_before_save = registry.get(&editor_mode_session_id)?;
    let mut document_value = session_before_save.document_value.clone();
    apply_non_document_value_transactions(&mut document_value, &session_before_save)?;

    let scene_path = session_before_save
        .root_path
        .join("scenes")
        .join(&session_before_save.scene_id)
        .join("scene.yml");
    let next_text = serde_yaml::to_string(&document_value).map_err(|error| {
        format!(
            "failed to serialize editor-mode scene document `{}`: {error}",
            scene_path.display()
        )
    })?;
    std::fs::write(&scene_path, next_text).map_err(|error| {
        format!(
            "failed to save editor-mode scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    let session = registry.update(&editor_mode_session_id, |session| {
        session.document_value = document_value;
        let mut snapshot = document_editor_snapshot(
            session.mod_id.clone(),
            &session.root_path,
            session.scene_id.clone(),
        )?;
        snapshot.canvas_kind = session.snapshot.canvas_kind;
        session.snapshot = enrich_snapshot_with_editor_control_state(
            snapshot,
            session.selected_entity_id.clone(),
            session.selected_ui_node.clone(),
            session.tool,
            Default::default(),
        );
        session.dirty = false;
        session.active_interaction = None;
        session.transactions.clear();
        session.bump_revision();
        session.saved_revision = session.revision;
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor-mode scene document saved.".to_owned()),
    })
}

pub async fn discard_editor_mode_session_changes(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
) -> Result<EditorFrameResultDto, String> {
    let current = registry.get(&editor_mode_session_id)?;
    let scene_path = current
        .root_path
        .join("scenes")
        .join(&current.scene_id)
        .join("scene.yml");
    let text = std::fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "failed to read scene document `{}` during discard: {error}",
            scene_path.display()
        )
    })?;
    let document_value = serde_yaml::from_str::<serde_yaml::Value>(&text).map_err(|error| {
        format!(
            "failed to parse scene document `{}` during discard: {error}",
            scene_path.display()
        )
    })?;
    let snapshot = document_editor_snapshot_from_value(
        current.mod_id.clone(),
        current.scene_id.clone(),
        &document_value,
    )?;

    let session = registry.update(&editor_mode_session_id, |session| {
        session.document_value = document_value;
        session.snapshot = enrich_snapshot_with_editor_control_state(
            snapshot,
            session.selected_entity_id.clone(),
            session.selected_ui_node.clone(),
            session.tool,
            Default::default(),
        );
        session.active_interaction = None;
        session.transactions.clear();
        session.dirty = false;
        session.bump_revision();
        session.saved_revision = session.revision;
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor-mode scene changes discarded.".to_owned()),
    })
}

fn apply_non_document_value_transactions(
    document: &mut serde_yaml::Value,
    session: &EditorModeSession,
) -> Result<(), String> {
    for transaction in &session.transactions.done {
        for fragment in &transaction.fragments {
            match fragment {
                EditorTransactionFragment::Transform2 {
                    entity_id, after, ..
                } => patch_entity_transform_2d(document, entity_id, after)?,
                EditorTransactionFragment::PrefabOverride {
                    entity_id,
                    target,
                    after,
                    ..
                } => patch_prefab_override(document, entity_id, target, after.clone())?,
                EditorTransactionFragment::SetUiNodeProperty { .. }
                | EditorTransactionFragment::UiDocumentValue { .. } => {}
            }
        }
    }
    Ok(())
}
