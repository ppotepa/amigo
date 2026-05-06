use std::collections::BTreeMap;

use tauri::AppHandle;

use crate::cache::root::EditorPaths;
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::document_commands::{
    apply_document_prefab_override, apply_document_transform_2d,
};
use crate::editor_mode::document_snapshot::document_editor_snapshot;

use super::dto::{EditorFrameResultDto, EditorTransform2Dto};
use super::gizmos::enrich_snapshot_with_editor_state;
use super::renderer::render_editor_mode_frame;
use super::session::{EditorModeSession, EditorModeSessionRegistry};
use super::transaction::EditorTransactionFragment;

#[derive(Debug, Clone)]
pub enum EditorDocumentPatchOperation {
    SetTransform2 {
        entity_id: String,
        transform: EditorTransform2Dto,
    },
    SetPrefabOverride {
        entity_id: String,
        target: String,
        value: serde_yaml::Value,
    },
}

#[derive(Debug, Clone)]
pub struct EditorDocumentPatchPlan {
    pub operations: Vec<EditorDocumentPatchOperation>,
}

pub async fn save_editor_mode_session_changes(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
) -> Result<EditorFrameResultDto, String> {
    let session_before_save = registry.get(&editor_mode_session_id)?;
    let plan = build_document_patch_plan(&session_before_save);

    let mut diagnostics = Vec::new();
    for operation in &plan.operations {
        match operation {
            EditorDocumentPatchOperation::SetTransform2 {
                entity_id,
                transform,
            } => {
                match apply_document_transform_2d(
                    session_before_save.mod_id.clone(),
                    &session_before_save.root_path,
                    session_before_save.scene_id.clone(),
                    entity_id.clone(),
                    transform.clone(),
                ) {
                    Ok(result) if result.ok => diagnostics.extend(result.diagnostics),
                    Ok(result) => {
                        diagnostics.extend(result.diagnostics);
                        return save_failure_response(
                            app,
                            paths,
                            registry,
                            editor_mode_session_id,
                            diagnostics,
                            result.message.unwrap_or_else(|| {
                                format!("Failed to apply transform2 patch for `{entity_id}`.")
                            }),
                        )
                        .await;
                    }
                    Err(error) => {
                        diagnostics.push(EditorDiagnosticDto {
                            level: DiagnosticLevel::Error,
                            code: error
                                .split(':')
                                .next()
                                .unwrap_or("EDITOR_DOCUMENT_PATCH_FAILED")
                                .to_owned(),
                            message: error,
                            path: None,
                        });
                        return save_failure_response(
                            app,
                            paths,
                            registry,
                            editor_mode_session_id,
                            diagnostics,
                            format!("Failed to apply transform2 patch for `{entity_id}`."),
                        )
                        .await;
                    }
                }
            }
            EditorDocumentPatchOperation::SetPrefabOverride {
                entity_id,
                target,
                value,
            } => {
                match apply_document_prefab_override(
                    session_before_save.mod_id.clone(),
                    &session_before_save.root_path,
                    session_before_save.scene_id.clone(),
                    entity_id.clone(),
                    target.clone(),
                    value.clone(),
                ) {
                    Ok(result) if result.ok => diagnostics.extend(result.diagnostics),
                    Ok(result) => {
                        diagnostics.extend(result.diagnostics);
                        return save_failure_response(
                            app,
                            paths,
                            registry,
                            editor_mode_session_id,
                            diagnostics,
                            result.message.unwrap_or_else(|| {
                                format!("Failed to apply prefab override for `{entity_id}` target `{target}`.")
                            }),
                        )
                        .await;
                    }
                    Err(error) => {
                        diagnostics.push(EditorDiagnosticDto {
                            level: DiagnosticLevel::Error,
                            code: error
                                .split(':')
                                .next()
                                .unwrap_or("EDITOR_DOCUMENT_PATCH_FAILED")
                                .to_owned(),
                            message: error,
                            path: None,
                        });
                        return save_failure_response(
                            app,
                            paths,
                            registry,
                            editor_mode_session_id,
                            diagnostics,
                            format!("Failed to apply prefab override for `{entity_id}` target `{target}`."),
                        )
                        .await;
                    }
                }
            }
        }
    }

    let session = registry.update(&editor_mode_session_id, |session| {
        let mut snapshot = document_editor_snapshot(
            session.mod_id.clone(),
            &session.root_path,
            session.scene_id.clone(),
        )?;
        snapshot.canvas_kind = session.snapshot.canvas_kind;
        session.dirty = false;
        session.snapshot = enrich_snapshot_with_editor_state(
            snapshot,
            session.selected_entity_id.clone(),
            session.tool,
        );
        session.active_interaction = None;
        session.transactions.clear();
        session.diagnostics = diagnostics.clone();
        session.bump_revision();
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor mode session saved.".to_owned()),
    })
}

pub async fn discard_editor_mode_session_changes(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
) -> Result<EditorFrameResultDto, String> {
    let session = registry.update(&editor_mode_session_id, |session| {
        let mut snapshot = document_editor_snapshot(
            session.mod_id.clone(),
            &session.root_path,
            session.scene_id.clone(),
        )?;
        snapshot.canvas_kind = session.snapshot.canvas_kind;
        session.snapshot = enrich_snapshot_with_editor_state(
            snapshot,
            session.selected_entity_id.clone(),
            session.tool,
        );
        session.active_interaction = None;
        session.transactions.clear();
        session.dirty = false;
        session.bump_revision();
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor mode changes discarded.".to_owned()),
    })
}

fn build_document_patch_plan(session: &EditorModeSession) -> EditorDocumentPatchPlan {
    let mut transforms = BTreeMap::<String, EditorTransform2Dto>::new();
    let mut operations = Vec::<EditorDocumentPatchOperation>::new();

    for transaction in &session.transactions.done {
        for fragment in &transaction.fragments {
            match fragment {
                EditorTransactionFragment::Transform2 {
                    entity_id, after, ..
                } => {
                    transforms.insert(entity_id.clone(), after.clone());
                }
                EditorTransactionFragment::PrefabOverride {
                    entity_id,
                    target,
                    after,
                    ..
                } => {
                    operations.push(EditorDocumentPatchOperation::SetPrefabOverride {
                        entity_id: entity_id.clone(),
                        target: target.clone(),
                        value: after.clone(),
                    });
                }
            }
        }
    }

    operations.extend(transforms.into_iter().map(|(entity_id, transform)| {
        EditorDocumentPatchOperation::SetTransform2 {
            entity_id,
            transform,
        }
    }));

    EditorDocumentPatchPlan { operations }
}

async fn save_failure_response(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
    diagnostics: Vec<EditorDiagnosticDto>,
    message: String,
) -> Result<EditorFrameResultDto, String> {
    let session = registry.update(&editor_mode_session_id, |session| {
        session.diagnostics = diagnostics.clone();
        session.dirty = session.transactions.is_dirty();
        session.bump_revision();
        Ok(())
    })?;
    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: false,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some(message),
    })
}
