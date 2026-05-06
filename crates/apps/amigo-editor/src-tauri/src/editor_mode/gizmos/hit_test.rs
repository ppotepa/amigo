use crate::editor_mode::controls::point_in_gizmo_hit_shape;
use crate::editor_mode::dto::{
    EditorGizmoHandleKindDto, EditorGizmoHitShapeDto, EditorSceneObjectDto, EditorSceneSnapshotDto,
    EditorUiNodeSelectionDto,
};

use super::builders::{point_in_bounds, selectable_bounds};

#[derive(Debug, Clone)]
pub struct EditorGizmoHandleHit {
    pub gizmo_id: String,
    pub entity_id: Option<String>,
    pub handle_id: String,
    pub handle_kind: EditorGizmoHandleKindDto,
}

#[derive(Debug, Clone)]
pub enum EditorPointerHitTarget {
    GizmoHandle(EditorGizmoHandleHit),
    UiNode(EditorUiNodeSelectionDto),
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

    if let Some(selected_ui_node) = snapshot.selection.selected_ui_node.as_ref() {
        if snapshot
            .ui_nodes
            .iter()
            .find(|node| {
                node.entity_id == selected_ui_node.entity_id
                    && node.component_index == selected_ui_node.component_index
                    && node.node_path == selected_ui_node.node_path
            })
            .map(|node| point_in_bounds(x, y, &node.bounds_2))
            .unwrap_or(false)
        {
            return EditorPointerHitTarget::UiNode(selected_ui_node.clone());
        }
    }

    if let Some(ui_node) = hit_test_snapshot_ui_node(snapshot, x, y) {
        return EditorPointerHitTarget::UiNode(ui_node);
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

pub fn hit_test_snapshot_ui_node(
    snapshot: &EditorSceneSnapshotDto,
    x: f32,
    y: f32,
) -> Option<EditorUiNodeSelectionDto> {
    snapshot
        .ui_nodes
        .iter()
        .rev()
        .find(|node| {
            node.visible && node.selectable && !node.locked && point_in_bounds(x, y, &node.bounds_2)
        })
        .map(|node| EditorUiNodeSelectionDto {
            entity_id: node.entity_id.clone(),
            component_index: node.component_index,
            node_path: node.node_path.clone(),
        })
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
                    gizmo_id: gizmo.id.clone(),
                    entity_id: gizmo.entity_id.clone(),
                    handle_id: handle.id.clone(),
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
    point_in_gizmo_hit_shape(x, y, shape)
}
