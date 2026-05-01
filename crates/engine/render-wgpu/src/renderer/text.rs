use crate::renderer::*;
use crate::ui_overlay::UiTextAnchor;

pub(crate) fn append_text_screen_space_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    content: &str,
    rect: crate::ui_overlay::UiRect,
    font_size: f32,
    color: ColorRgba,
    anchor: UiTextAnchor,
    word_wrap: bool,
    fit_to_width: bool,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let (effective_font_size, lines) =
        layout_ui_text_lines(content, rect.width, font_size, word_wrap, fit_to_width);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    let line_height = effective_font_size * 1.2;
    let line_widths = lines
        .iter()
        .map(|line| line.chars().count() as f32 * advance)
        .collect::<Vec<_>>();
    let text_width = line_widths.iter().copied().fold(0.0, f32::max);
    let text_height = line_height * lines.len().max(1) as f32;

    let origin_x = match anchor {
        UiTextAnchor::TopLeft => rect.x,
        UiTextAnchor::Center => rect.x + (rect.width - text_width).max(0.0) * 0.5,
    };
    let origin_y = match anchor {
        UiTextAnchor::TopLeft => rect.y,
        UiTextAnchor::Center => rect.y + (rect.height - text_height).max(0.0) * 0.5,
    };

    for (line_index, line) in lines.iter().enumerate() {
        let line_origin_y = origin_y + line_index as f32 * line_height;
        let line_origin_x = match anchor {
            UiTextAnchor::TopLeft => origin_x,
            UiTextAnchor::Center => rect.x + (rect.width - line_widths[line_index]).max(0.0) * 0.5,
        };
        for (index, ch) in line.chars().enumerate() {
            let rows = glyph_rows(ch);
            let glyph_origin_x = line_origin_x + index as f32 * advance;
            for (row_index, row_bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if row_bits & (1 << (4 - column)) == 0 {
                        continue;
                    }

                    let min = Vec2::new(
                        glyph_origin_x + column as f32 * pixel_size,
                        line_origin_y + row_index as f32 * pixel_size,
                    );
                    let max = Vec2::new(min.x + pixel_size, min.y + pixel_size);
                    let quad = [
                        ndc_from_ui_screen(min, viewport),
                        ndc_from_ui_screen(Vec2::new(max.x, min.y), viewport),
                        ndc_from_ui_screen(max, viewport),
                        ndc_from_ui_screen(Vec2::new(min.x, max.y), viewport),
                    ];
                    push_quad(vertices, quad[0], quad[1], quad[2], quad[3], color);
                }
            }
        }
    }
}

fn layout_ui_text_lines(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> (f32, Vec<String>) {
    let mut effective_font_size = font_size.max(8.0);
    if fit_to_width && !word_wrap && max_width > 0.0 {
        let width = measure_ui_text_line_width(content, effective_font_size);
        if width > max_width {
            effective_font_size = (effective_font_size * (max_width / width))
                .max(8.0)
                .min(effective_font_size);
        }
    }

    let lines = if word_wrap && max_width > 0.0 {
        wrap_ui_text_lines(content, effective_font_size, max_width)
    } else {
        content.split('\n').map(|line| line.to_owned()).collect()
    };

    (
        effective_font_size,
        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        },
    )
}

fn wrap_ui_text_lines(content: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if measure_ui_text_line_width(&candidate, font_size) <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }

            if measure_ui_text_line_width(word, font_size) <= max_width {
                current = word.to_owned();
                continue;
            }

            let mut fragment = String::new();
            for ch in word.chars() {
                let candidate = format!("{fragment}{ch}");
                if !fragment.is_empty()
                    && measure_ui_text_line_width(&candidate, font_size) > max_width
                {
                    lines.push(fragment.clone());
                    fragment.clear();
                }
                fragment.push(ch);
            }
            current = fragment;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn measure_ui_text_line_width(content: &str, font_size: f32) -> f32 {
    let effective_font_size = font_size.max(8.0);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    content.chars().count() as f32 * advance
}
