use crate::renderer::*;

pub(crate) fn append_ui_overlay_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    primitives: &[UiDrawPrimitive],
) {
    for primitive in primitives {
        match primitive {
            UiDrawPrimitive::Quad { rect, color } => {
                append_ui_quad_vertices(vertices, viewport, *rect, *color);
            }
            UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: _font,
                anchor,
                word_wrap,
                fit_to_width,
            } => append_text_screen_space_vertices(
                vertices,
                viewport,
                content,
                *rect,
                *font_size,
                *color,
                *anchor,
                *word_wrap,
                *fit_to_width,
                false,
            ),
            UiDrawPrimitive::ProgressBar {
                rect,
                value,
                background,
                foreground,
            } => append_progress_bar_vertices(
                vertices,
                viewport,
                *rect,
                *value,
                *background,
                *foreground,
            ),
        }
    }
}

fn append_ui_quad_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    rect: crate::ui_overlay::UiRect,
    color: ColorRgba,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let points = [
        ndc_from_ui_screen(Vec2::new(rect.x, rect.y), viewport),
        ndc_from_ui_screen(Vec2::new(rect.x + rect.width, rect.y), viewport),
        ndc_from_ui_screen(
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
            viewport,
        ),
        ndc_from_ui_screen(Vec2::new(rect.x, rect.y + rect.height), viewport),
    ];
    push_quad(vertices, points[0], points[1], points[2], points[3], color);
}

fn append_progress_bar_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    rect: crate::ui_overlay::UiRect,
    value: f32,
    background: ColorRgba,
    foreground: ColorRgba,
) {
    append_ui_quad_vertices(vertices, viewport, rect, background);

    let inset = rect.height.min(4.0).max(2.0);
    let inner = rect.inset(inset);
    let clamped = value.clamp(0.0, 1.0);
    let fill_rect =
        crate::ui_overlay::UiRect::new(inner.x, inner.y, inner.width * clamped, inner.height);
    append_ui_quad_vertices(vertices, viewport, fill_rect, foreground);
}

