use crate::editor_mode::dto::{EditorGizmoHitShapeDto, EditorGizmoRectDto};

pub fn point_in_gizmo_hit_shape(x: f32, y: f32, shape: &EditorGizmoHitShapeDto) -> bool {
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
            let d = distance(x, y, center.x, center.y);
            d >= *inner_radius && d <= *outer_radius
        }
    }
}

fn point_in_rect(x: f32, y: f32, rect: &EditorGizmoRectDto) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

fn distance(x: f32, y: f32, cx: f32, cy: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
}
