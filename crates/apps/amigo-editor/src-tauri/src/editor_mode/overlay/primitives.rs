use super::super::coordinates::{
    EditorCoordinateMapper, EditorFramePoint, EditorScenePoint, EditorSceneRect,
};
use super::super::dto::{
    EditorGizmoPointDto, EditorGizmoPrimitiveDto, EditorGizmoRectDto, EditorGizmoToneDto,
};

const BASE_STROKE_WIDTH: f32 = 2.0;
const HANDLE_STROKE_WIDTH: f32 = 1.5;
const ARROW_HEAD_LENGTH: f32 = 14.0;
const ARROW_HEAD_HALF_WIDTH: f32 = 6.0;

pub fn append_primitive(
    svg: &mut String,
    primitive: &EditorGizmoPrimitiveDto,
    mapper: EditorCoordinateMapper,
) {
    match primitive {
        EditorGizmoPrimitiveDto::Line2D { from, to, tone } => {
            let from = scene_to_frame_point(from, mapper);
            let to = scene_to_frame_point(to, mapper);
            svg.push_str(&format!(
                r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="{:.2}"/>"#,
                from.x,
                from.y,
                to.x,
                to.y,
                tone_color(*tone),
                tone_stroke_width(*tone),
            ));
        }
        EditorGizmoPrimitiveDto::Arrow2D { from, to, tone } => {
            append_arrow(svg, from, to, *tone, mapper);
        }
        EditorGizmoPrimitiveDto::Rect2D { rect, tone } => {
            let rect = scene_to_frame_rect(rect, mapper);
            let dash = if *tone == EditorGizmoToneDto::Selection {
                r#" stroke-dasharray="7 5""#
            } else {
                ""
            };
            svg.push_str(&format!(
                r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" fill-opacity="{:.2}" stroke="{}" stroke-width="{:.2}"{} />"#,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                rect_fill(*tone),
                rect_fill_opacity(*tone),
                tone_color(*tone),
                if rect.width <= 12.0 || rect.height <= 12.0 {
                    HANDLE_STROKE_WIDTH.max(tone_stroke_width(*tone))
                } else {
                    tone_stroke_width(*tone)
                },
                dash,
            ));
        }
        EditorGizmoPrimitiveDto::Circle2D {
            center,
            radius,
            tone,
        } => {
            let center = scene_to_frame_point(center, mapper);
            let radius = radius * mapper.camera.zoom.max(0.0001);
            svg.push_str(&format!(
                r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" stroke="{}" stroke-width="{:.2}" fill="{}" fill-opacity="0.22"/>"#,
                center.x,
                center.y,
                radius,
                tone_color(*tone),
                BASE_STROKE_WIDTH,
                tone_color(*tone),
            ));
        }
        EditorGizmoPrimitiveDto::Ring2D {
            center,
            inner_radius,
            outer_radius,
            tone,
        } => {
            let center = scene_to_frame_point(center, mapper);
            let zoom = mapper.camera.zoom.max(0.0001);
            let stroke_width = ((outer_radius - inner_radius) * zoom).max(1.0);
            let radius = ((inner_radius + outer_radius) / 2.0) * zoom;
            svg.push_str(&format!(
                r#"<circle cx="{:.2}" cy="{:.2}" r="{:.2}" stroke="{}" stroke-width="{:.2}" stroke-opacity="0.92"/>"#,
                center.x,
                center.y,
                radius,
                tone_color(*tone),
                stroke_width,
            ));
        }
    }
}

fn append_arrow(
    svg: &mut String,
    from: &EditorGizmoPointDto,
    to: &EditorGizmoPointDto,
    tone: EditorGizmoToneDto,
    mapper: EditorCoordinateMapper,
) {
    let from = scene_to_frame_point(from, mapper);
    let to = scene_to_frame_point(to, mapper);
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 0.001 {
        return;
    }

    let ux = dx / length;
    let uy = dy / length;
    let bx = to.x - ux * ARROW_HEAD_LENGTH;
    let by = to.y - uy * ARROW_HEAD_LENGTH;
    let px = -uy;
    let py = ux;
    let left_x = bx + px * ARROW_HEAD_HALF_WIDTH;
    let left_y = by + py * ARROW_HEAD_HALF_WIDTH;
    let right_x = bx - px * ARROW_HEAD_HALF_WIDTH;
    let right_y = by - py * ARROW_HEAD_HALF_WIDTH;
    let color = tone_color(tone);

    svg.push_str(&format!(
        r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="{:.2}"/>"#,
        from.x,
        from.y,
        bx,
        by,
        color,
        tone_stroke_width(tone),
    ));
    svg.push_str(&format!(
        r#"<polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="{}" stroke="{}" stroke-width="1"/>"#,
        to.x,
        to.y,
        left_x,
        left_y,
        right_x,
        right_y,
        color,
        color,
    ));
}

fn scene_to_frame_point(
    point: &EditorGizmoPointDto,
    mapper: EditorCoordinateMapper,
) -> EditorFramePoint {
    mapper.scene_to_frame_point(EditorScenePoint {
        x: point.x,
        y: point.y,
    })
}

fn scene_to_frame_rect(
    rect: &EditorGizmoRectDto,
    mapper: EditorCoordinateMapper,
) -> super::super::coordinates::EditorFrameRect {
    mapper.scene_to_frame_rect(EditorSceneRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    })
}

fn tone_color(tone: EditorGizmoToneDto) -> &'static str {
    match tone {
        EditorGizmoToneDto::Neutral => "#e2e8f0",
        EditorGizmoToneDto::Selection => "#38bdf8",
        EditorGizmoToneDto::X => "#f87171",
        EditorGizmoToneDto::Y => "#4ade80",
        EditorGizmoToneDto::Rotation => "#c084fc",
        EditorGizmoToneDto::Scale => "#facc15",
        EditorGizmoToneDto::Warning => "#fb923c",
        EditorGizmoToneDto::HoverX => "#f59e0b",
        EditorGizmoToneDto::HoverY => "#a3e635",
        EditorGizmoToneDto::Center => "#38bdf8",
        EditorGizmoToneDto::CenterHover => "#facc15",
        EditorGizmoToneDto::CenterActive => "#facc15",
        EditorGizmoToneDto::RotationHover => "#eab308",
        EditorGizmoToneDto::RotationActive => "#facc15",
        EditorGizmoToneDto::ScaleHover => "#facc15",
        EditorGizmoToneDto::ScaleActive => "#facc15",
        EditorGizmoToneDto::Active => "#facc15",
    }
}

fn tone_stroke_width(tone: EditorGizmoToneDto) -> f32 {
    match tone {
        EditorGizmoToneDto::HoverX
        | EditorGizmoToneDto::HoverY
        | EditorGizmoToneDto::CenterHover
        | EditorGizmoToneDto::RotationHover
        | EditorGizmoToneDto::ScaleHover => BASE_STROKE_WIDTH + 1.0,
        EditorGizmoToneDto::CenterActive
        | EditorGizmoToneDto::RotationActive
        | EditorGizmoToneDto::ScaleActive
        | EditorGizmoToneDto::Active => BASE_STROKE_WIDTH + 1.5,
        _ => BASE_STROKE_WIDTH,
    }
}

fn rect_fill(tone: EditorGizmoToneDto) -> &'static str {
    match tone {
        EditorGizmoToneDto::CenterHover
        | EditorGizmoToneDto::CenterActive
        | EditorGizmoToneDto::ScaleHover
        | EditorGizmoToneDto::ScaleActive => "#facc15",
        _ => "none",
    }
}

fn rect_fill_opacity(tone: EditorGizmoToneDto) -> f32 {
    match tone {
        EditorGizmoToneDto::CenterActive | EditorGizmoToneDto::ScaleActive => 0.28,
        EditorGizmoToneDto::CenterHover | EditorGizmoToneDto::ScaleHover => 0.16,
        _ => 0.0,
    }
}
