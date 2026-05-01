
pub fn normalize_curve_points(points: &[UiCurvePoint]) -> Vec<UiCurvePoint> {
    let mut normalized = if points.is_empty() {
        default_curve_points()
    } else {
        points
            .iter()
            .map(|point| UiCurvePoint {
                t: point.t.clamp(0.0, 1.0),
                value: point.value.clamp(0.0, 1.0),
            })
            .collect::<Vec<_>>()
    };
    normalized.sort_by(|a, b| a.t.total_cmp(&b.t));

    while normalized.len() < 4 {
        let t = (normalized.len() as f32 / 3.0).clamp(0.0, 1.0);
        normalized.push(UiCurvePoint::new(t, t));
        normalized.sort_by(|a, b| a.t.total_cmp(&b.t));
    }

    normalized
}

pub fn default_curve_points() -> Vec<UiCurvePoint> {
    vec![
        UiCurvePoint::new(0.0, 0.0),
        UiCurvePoint::new(1.0 / 3.0, 1.0 / 3.0),
        UiCurvePoint::new(2.0 / 3.0, 2.0 / 3.0),
        UiCurvePoint::new(1.0, 1.0),
    ]
}

pub fn curve_points_from_values(values: &[f32]) -> Vec<UiCurvePoint> {
    if values.is_empty() {
        return default_curve_points();
    }
    let denominator = (values.len().saturating_sub(1)).max(1) as f32;
    normalize_curve_points(
        &values
            .iter()
            .enumerate()
            .map(|(index, value)| UiCurvePoint::new(index as f32 / denominator, *value))
            .collect::<Vec<_>>(),
    )
}

pub fn format_curve_points(points: &[UiCurvePoint]) -> String {
    normalize_curve_points(points)
        .iter()
        .map(|point| format!("{:.4}:{:.4}", point.t, point.value))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn curve_editor_edit_from_mouse(
    rect: UiRect,
    points: &[UiCurvePoint],
    mouse_x: f32,
    mouse_y: f32,
) -> Option<UiCurveEdit> {
    if rect.width <= f32::EPSILON || rect.height <= f32::EPSILON {
        return None;
    }

    let mut points = normalize_curve_points(points);
    let t = ((mouse_x - rect.x) / rect.width).clamp(0.0, 1.0);
    let value = (1.0 - ((mouse_y - rect.y) / rect.height)).clamp(0.0, 1.0);
    let point_index = nearest_curve_point_index(&points, t);
    points[point_index] = UiCurvePoint::new(t, value);
    points.sort_by(|a, b| a.t.total_cmp(&b.t));
    let point_index = nearest_curve_point_index(&points, t);
    let point = points[point_index];

    Some(UiCurveEdit {
        point_index,
        point,
        points,
    })
}

fn nearest_curve_point_index(points: &[UiCurvePoint], t: f32) -> usize {
    points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (a.t - t).abs().total_cmp(&(b.t - t).abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

