use crate::editor_mode::dto::{
    EditorGizmoHandleKindDto, EditorGizmoHitShapeDto, EditorGizmoRectDto, EditorSceneObjectDto,
    EditorSceneSnapshotDto,
};

use super::builders::{point_in_bounds, selectable_bounds};

#[derive(Debug, Clone)]
pub struct EditorGizmoHandleHit {
    pub entity_id: Option<String>,
    pub handle_kind: EditorGizmoHandleKindDto,
}

#[derive(Debug, Clone)]
pub enum EditorPointerHitTarget {
    GizmoHandle(EditorGizmoHandleHit),
    Entity(String),
    Empty,
}

pub fn hit_test_snapshot_entity(
    snapshot: &EditorSceneSnapshotDto,
    x: f32,
    y: f32,
) -> Option<String> {
    snapshot
        .objects
        .iter()
        .rev()
        .find(|object| is_entity_hit_target(object, x, y))
        .map(|object| object.entity_id.clone())
}

pub fn hit_test_editor_snapshot(
    snapshot: &EditorSceneSnapshotDto,
    x: f32,
    y: f32,
) -> EditorPointerHitTarget {
    if let Some(hit) = hit_test_gizmo_handles(snapshot, x, y) {
        return EditorPointerHitTarget::GizmoHandle(hit);
    }

    if let Some(entity_id) = snapshot.selection.selected_entity_ids.first() {
        if snapshot
            .objects
            .iter()
            .find(|object| object.entity_id == *entity_id)
            .and_then(selectable_bounds)
            .map(|bounds| point_in_bounds(x, y, bounds))
            .unwrap_or(false)
        {
            return EditorPointerHitTarget::Entity(entity_id.clone());
        }
    }

    hit_test_snapshot_entity(snapshot, x, y)
        .map(EditorPointerHitTarget::Entity)
        .unwrap_or(EditorPointerHitTarget::Empty)
}

pub fn hit_test_gizmo_handles(
    snapshot: &EditorSceneSnapshotDto,
    x: f32,
    y: f32,
) -> Option<EditorGizmoHandleHit> {
    snapshot.gizmos.iter().rev().find_map(|gizmo| {
        gizmo.handles.iter().rev().find_map(|handle| {
            if point_in_hit_shape(x, y, &handle.hit_shape) {
                return Some(EditorGizmoHandleHit {
                    entity_id: gizmo.entity_id.clone(),
                    handle_kind: handle.kind,
                });
            }
            None
        })
    })
}

fn is_entity_hit_target(object: &EditorSceneObjectDto, x: f32, y: f32) -> bool {
    if !object.visible || !object.selectable || object.locked {
        return false;
    }
    selectable_bounds(object)
        .map(|bounds| point_in_bounds(x, y, bounds))
        .unwrap_or(false)
}

fn point_in_hit_shape(x: f32, y: f32, shape: &EditorGizmoHitShapeDto) -> bool {
    match shape {
        EditorGizmoHitShapeDto::Rect2D { rect } => point_in_rect(x, y, rect),
        EditorGizmoHitShapeDto::Circle2D { center, radius } => {
            distance(x, y, center.x, center.y) <= *radius
        }
        EditorGizmoHitShapeDto::Ring2D {
            center,
            inner_radius,
            outer_radius,
        } => {
            let distance = distance(x, y, center.x, center.y);
            distance >= *inner_radius && distance <= *outer_radius
        }
    }
}

fn point_in_rect(x: f32, y: f32, rect: &EditorGizmoRectDto) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

fn distance(x: f32, y: f32, center_x: f32, center_y: f32) -> f32 {
    ((x - center_x).powi(2) + (y - center_y).powi(2)).sqrt()
}
