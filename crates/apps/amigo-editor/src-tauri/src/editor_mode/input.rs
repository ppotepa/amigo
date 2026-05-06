use tauri::AppHandle;

use crate::cache::root::EditorPaths;
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

use super::dto::{
    EditorFrameResultDto, EditorGizmoHandleKindDto, EditorPointerEventDto, EditorToolDto,
    EditorTransform2Dto,
};
use super::gizmos::{
    EditorPointerHitTarget, enrich_snapshot_with_editor_state, hit_test_editor_snapshot,
};
use super::renderer::render_editor_mode_frame;
use super::session::{
    EditorActiveInteraction, EditorActiveInteractionKind, EditorModeSession,
    EditorModeSessionRegistry,
};
use super::transaction::{
    EditorTransaction, EditorTransactionFragment, apply_snapshot_transform_2,
};

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

fn handle_pointer_down(session: &mut EditorModeSession, event: &EditorPointerEventDto) {
    session.active_interaction = None;

    if matches!(session.tool, EditorToolDto::Pan) {
        session.active_interaction = Some(EditorActiveInteraction {
            kind: EditorActiveInteractionKind::PanViewport,
            entity_id: None,
            start_pointer_x: event.scene_x(),
            start_pointer_y: event.scene_y(),
            start_transform_2: None,
            changed_entities: Vec::new(),
        });
        session.bump_revision();
        return;
    }

    match hit_test_editor_snapshot(&session.snapshot, event.scene_x(), event.scene_y()) {
        EditorPointerHitTarget::GizmoHandle(hit) => {
            let entity_id = hit.entity_id.clone();
            session.selected_entity_id = entity_id.clone();
            session.snapshot = enrich_snapshot_with_editor_state(
                session.snapshot.clone(),
                session.selected_entity_id.clone(),
                session.tool,
            );

            if let Some(entity_id) = entity_id {
                session.active_interaction =
                    move_interaction_for_handle(session, &entity_id, hit.handle_kind, event)
                        .or_else(|| {
                            Some(select_interaction(
                                Some(entity_id),
                                event.scene_x(),
                                event.scene_y(),
                            ))
                        });
            }
            push_pointer_diagnostic(session, event, "hit gizmo handle");
        }
        EditorPointerHitTarget::Entity(entity_id) => {
            session.selected_entity_id = Some(entity_id.clone());
            session.snapshot = enrich_snapshot_with_editor_state(
                session.snapshot.clone(),
                session.selected_entity_id.clone(),
                session.tool,
            );
            session.active_interaction =
                move_interaction_for_entity_body(session, &entity_id, event).or_else(|| {
                    Some(select_interaction(
                        Some(entity_id.clone()),
                        event.scene_x(),
                        event.scene_y(),
                    ))
                });
            push_pointer_diagnostic(session, event, &format!("hit entity `{entity_id}`"));
        }
        EditorPointerHitTarget::Empty => {
            session.selected_entity_id = None;
            session.snapshot =
                enrich_snapshot_with_editor_state(session.snapshot.clone(), None, session.tool);
            push_pointer_diagnostic(session, event, "hit empty scene");
        }
    }

    session.bump_revision();
}

fn handle_pointer_move(session: &mut EditorModeSession, event: &EditorPointerEventDto) {
    let Some(interaction) = session.active_interaction.clone() else {
        return;
    };

    match interaction.kind {
        EditorActiveInteractionKind::MoveEntity2D
        | EditorActiveInteractionKind::MoveAxisX
        | EditorActiveInteractionKind::MoveAxisY => {
            let Some(entity_id) = interaction.entity_id.as_deref() else {
                return;
            };
            let Some(start_transform) = interaction.start_transform_2.clone() else {
                return;
            };

            let mut dx = event.scene_x() - interaction.start_pointer_x;
            let mut dy = event.scene_y() - interaction.start_pointer_y;
            if matches!(interaction.kind, EditorActiveInteractionKind::MoveAxisX) {
                dy = 0.0;
            }
            if matches!(interaction.kind, EditorActiveInteractionKind::MoveAxisY) {
                dx = 0.0;
            }

            let next_transform = EditorTransform2Dto {
                x: start_transform.x + dx,
                y: start_transform.y + dy,
                ..start_transform
            };
            apply_snapshot_transform_2(&mut session.snapshot, entity_id, next_transform);
            session.snapshot = enrich_snapshot_with_editor_state(
                session.snapshot.clone(),
                Some(entity_id.to_owned()),
                session.tool,
            );

            if let Some(active) = session.active_interaction.as_mut() {
                if !active.changed_entities.iter().any(|id| id == entity_id) {
                    active.changed_entities.push(entity_id.to_owned());
                }
            }
            session.bump_revision();
        }
        EditorActiveInteractionKind::SelectEntity | EditorActiveInteractionKind::PanViewport => {
            session.bump_revision();
        }
    }
}

fn handle_pointer_up(session: &mut EditorModeSession) {
    let active = session.active_interaction.take();

    if let Some(interaction) = active {
        if matches!(
            interaction.kind,
            EditorActiveInteractionKind::MoveEntity2D
                | EditorActiveInteractionKind::MoveAxisX
                | EditorActiveInteractionKind::MoveAxisY
        ) {
            if let (Some(entity_id), Some(before)) = (
                interaction.entity_id.clone(),
                interaction.start_transform_2.clone(),
            ) {
                if let Some(after) = current_transform_2(session, &entity_id) {
                    if transform_changed(&before, &after) {
                        session.transactions.push(EditorTransaction {
                            label: format!("Move {entity_id}"),
                            changed_entities: vec![entity_id.clone()],
                            fragments: vec![EditorTransactionFragment::Transform2 {
                                entity_id: entity_id.clone(),
                                before,
                                after,
                            }],
                        });
                        session.dirty = session.transactions.is_dirty();
                    }
                }
            }
        }
    }

    session.snapshot = enrich_snapshot_with_editor_state(
        session.snapshot.clone(),
        session.selected_entity_id.clone(),
        session.tool,
    );
    session.bump_revision();
}

fn handle_pointer_cancel(session: &mut EditorModeSession) {
    let active = session.active_interaction.take();
    if let Some(interaction) = active {
        if let (Some(entity_id), Some(start_transform)) = (
            interaction.entity_id.as_deref(),
            interaction.start_transform_2.clone(),
        ) {
            apply_snapshot_transform_2(&mut session.snapshot, entity_id, start_transform);
        }
    }

    session.snapshot = enrich_snapshot_with_editor_state(
        session.snapshot.clone(),
        session.selected_entity_id.clone(),
        session.tool,
    );
    session.bump_revision();
}

fn select_interaction(
    entity_id: Option<String>,
    start_pointer_x: f32,
    start_pointer_y: f32,
) -> EditorActiveInteraction {
    EditorActiveInteraction {
        kind: EditorActiveInteractionKind::SelectEntity,
        entity_id,
        start_pointer_x,
        start_pointer_y,
        start_transform_2: None,
        changed_entities: Vec::new(),
    }
}

fn move_interaction_for_handle(
    session: &EditorModeSession,
    entity_id: &str,
    handle_kind: EditorGizmoHandleKindDto,
    event: &EditorPointerEventDto,
) -> Option<EditorActiveInteraction> {
    if !matches!(session.tool, EditorToolDto::Move) {
        return None;
    }

    let kind = match handle_kind {
        EditorGizmoHandleKindDto::AxisX => EditorActiveInteractionKind::MoveAxisX,
        EditorGizmoHandleKindDto::AxisY => EditorActiveInteractionKind::MoveAxisY,
        EditorGizmoHandleKindDto::PlaneXY | EditorGizmoHandleKindDto::Body => {
            EditorActiveInteractionKind::MoveEntity2D
        }
        _ => return None,
    };

    move_interaction(session, entity_id, kind, event)
}

fn move_interaction_for_entity_body(
    session: &EditorModeSession,
    entity_id: &str,
    event: &EditorPointerEventDto,
) -> Option<EditorActiveInteraction> {
    if !matches!(session.tool, EditorToolDto::Move) {
        return None;
    }
    move_interaction(
        session,
        entity_id,
        EditorActiveInteractionKind::MoveEntity2D,
        event,
    )
}

fn move_interaction(
    session: &EditorModeSession,
    entity_id: &str,
    kind: EditorActiveInteractionKind,
    event: &EditorPointerEventDto,
) -> Option<EditorActiveInteraction> {
    let object = session
        .snapshot
        .objects
        .iter()
        .find(|object| object.entity_id == entity_id)?;
    if !object.visible || object.locked || !object.movable {
        return None;
    }
    let start_transform = object.transform_2.clone()?;

    Some(EditorActiveInteraction {
        kind,
        entity_id: Some(entity_id.to_owned()),
        start_pointer_x: event.scene_x(),
        start_pointer_y: event.scene_y(),
        start_transform_2: Some(start_transform),
        changed_entities: Vec::new(),
    })
}

fn push_pointer_diagnostic(
    session: &mut EditorModeSession,
    event: &EditorPointerEventDto,
    hit_message: &str,
) {
    session.diagnostics.push(EditorDiagnosticDto {
        level: DiagnosticLevel::Info,
        code: "EDITOR_POINTER_DOWN".to_owned(),
        message: format!(
            "pointerDown scene=({:.2},{:.2}) frame=({:?},{:?}) tool={:?}; {hit_message}",
            event.scene_x(),
            event.scene_y(),
            event.frame_x(),
            event.frame_y(),
            session.tool,
        ),
        path: None,
    });
}

fn current_transform_2(
    session: &EditorModeSession,
    entity_id: &str,
) -> Option<EditorTransform2Dto> {
    session
        .snapshot
        .objects
        .iter()
        .find(|object| object.entity_id == entity_id)
        .and_then(|object| object.transform_2.clone())
}

fn transform_changed(before: &EditorTransform2Dto, after: &EditorTransform2Dto) -> bool {
    const EPSILON: f32 = 0.0001;
    (before.x - after.x).abs() > EPSILON
        || (before.y - after.y).abs() > EPSILON
        || (before.rotation - after.rotation).abs() > EPSILON
        || (before.scale_x - after.scale_x).abs() > EPSILON
        || (before.scale_y - after.scale_y).abs() > EPSILON
        || before.z_index != after.z_index
}
