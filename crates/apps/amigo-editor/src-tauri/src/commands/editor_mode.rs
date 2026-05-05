use tauri::State;

use crate::editor_mode::dto::{
    EditorCameraDto, EditorCommandDto, EditorCommandResultDto, EditorHitTestCandidateDto,
    EditorHitTestResultDto, EditorSceneCanvasKindDto, EditorSceneSnapshotDto,
    EditorViewportPointDto,
};
use crate::editor_mode::{
    apply_document_transform_2d, document_editor_snapshot, fallback_editor_snapshot,
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
        return match apply_document_transform_2d(
            session.mod_id,
            session.root_path,
            scene_id,
            entity_id,
            transform,
        ) {
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
                message: Some("Editor transform command failed.".to_owned()),
            }),
        };
    }

    let changed_entities = match &command {
        EditorCommandDto::MoveEntity2D { entity_id, .. } => vec![entity_id.clone()],
        EditorCommandDto::SetEntityTransform2D { entity_id, .. } => vec![entity_id.clone()],
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
