use crate::ui_overlay::{
    UiOverlayCurvePoint, UiOverlayNode, UiOverlayNodeKind,
};

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

pub(crate) fn default_child_width_for_column(
    node: &UiOverlayNode,
    content_width: f32,
    measured_width: f32,
) -> f32 {
    if matches!(
        node.kind,
        UiOverlayNodeKind::Text { .. } | UiOverlayNodeKind::Button { .. }
    ) && (node.style.fit_to_width || node.style.word_wrap)
    {
        return content_width.max(measured_width).max(0.0);
    }

    match node.kind {
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::GroupBox { .. }
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::TabView { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. }
        | UiOverlayNodeKind::CurveEditor { .. }
        | UiOverlayNodeKind::Spacer => content_width.max(measured_width),
        UiOverlayNodeKind::Text { .. } | UiOverlayNodeKind::Button { .. } => measured_width,
    }
}

pub(crate) fn default_child_height_for_row(
    node: &UiOverlayNode,
    content_height: f32,
    measured_height: f32,
) -> f32 {
    match node.kind {
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::GroupBox { .. }
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::Spacer => content_height.max(measured_height),
        UiOverlayNodeKind::Text { .. }
        | UiOverlayNodeKind::Button { .. }
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::TabView { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. }
        | UiOverlayNodeKind::CurveEditor { .. } => measured_height,
    }
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

pub(crate) fn resolve_screen_axis(
    start: Option<f32>,
    end: Option<f32>,
    viewport: f32,
    size: f32,
) -> f32 {
    if let Some(start) = start {
        start
    } else if let Some(end) = end {
        viewport - size - end
    } else {
        0.0
    }
}

