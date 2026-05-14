use crate::ui_overlay::{UiOverlayCurvePoint, UiOverlayNodeKind};

pub(crate) fn normalized_curve_points(points: &[UiOverlayCurvePoint]) -> Vec<UiOverlayCurvePoint> {
    let mut points = points
        .iter()
        .copied()
        .filter(|point| point.t.is_finite() && point.value.is_finite())
        .map(|point| UiOverlayCurvePoint {
            t: point.t.clamp(0.0, 1.0),
            value: point.value.clamp(0.0, 1.0),
        })
        .collect::<Vec<_>>();
    if points.is_empty() {
        points = vec![
            UiOverlayCurvePoint { t: 0.0, value: 0.0 },
            UiOverlayCurvePoint {
                t: 1.0 / 3.0,
                value: 1.0 / 3.0,
            },
            UiOverlayCurvePoint {
                t: 2.0 / 3.0,
                value: 2.0 / 3.0,
            },
            UiOverlayCurvePoint { t: 1.0, value: 1.0 },
        ];
    }
    points.sort_by(|a, b| a.t.total_cmp(&b.t));
    while points.len() < 4 {
        let t = (points.len() as f32 / 3.0).clamp(0.0, 1.0);
        points.push(UiOverlayCurvePoint { t, value: t });
        points.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
    points
}

pub(crate) fn kind_slug(kind: &UiOverlayNodeKind) -> &'static str {
    match kind {
        UiOverlayNodeKind::Panel => "panel",
        UiOverlayNodeKind::GroupBox { .. } => "group-box",
        UiOverlayNodeKind::Row => "row",
        UiOverlayNodeKind::Column => "column",
        UiOverlayNodeKind::Stack => "stack",
        UiOverlayNodeKind::Text { .. } => "text",
        UiOverlayNodeKind::Button { .. } => "button",
        UiOverlayNodeKind::ProgressBar { .. } => "progress-bar",
        UiOverlayNodeKind::Slider { .. } => "slider",
        UiOverlayNodeKind::Toggle { .. } => "toggle",
        UiOverlayNodeKind::OptionSet { .. } => "option-set",
        UiOverlayNodeKind::Dropdown { .. } => "dropdown",
        UiOverlayNodeKind::TabView { .. } => "tab-view",
        UiOverlayNodeKind::ColorPickerRgb { .. } => "color-picker-rgb",
        UiOverlayNodeKind::CurveEditor { .. } => "curve-editor",
        UiOverlayNodeKind::Spacer => "spacer",
    }
}
