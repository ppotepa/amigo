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
    notebook_ink: bool,
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
                    if notebook_ink {
                        append_notebook_ink_cell(
                            vertices,
                            viewport,
                            min,
                            pixel_size,
                            color,
                            ch,
                            index,
                            row_index,
                            column,
                        );
                        continue;
                    }

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

fn append_notebook_ink_cell(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    min: Vec2,
    pixel_size: f32,
    color: ColorRgba,
    ch: char,
    glyph_index: usize,
    row_index: usize,
    column: usize,
) {
    let cell_seed =
        ch as u32 ^ ((glyph_index as u32) << 4) ^ ((row_index as u32) << 9) ^ ((column as u32) << 14);
    let jitter_x = signed_unit_hash(cell_seed) * pixel_size * 0.08;
    let jitter_y = signed_unit_hash(cell_seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
        * pixel_size
        * 0.08;

    let s = pixel_size;
    let x = min.x + jitter_x;
    let y = min.y + jitter_y;

    append_notebook_ink_stroke(
        vertices,
        viewport,
        Vec2::new(x + s * 0.12, y + s * 0.28),
        Vec2::new(x + s * 0.78, y + s * 0.16),
        s * 0.13,
        color,
    );
    append_notebook_ink_stroke(
        vertices,
        viewport,
        Vec2::new(x + s * 0.20, y + s * 0.56),
        Vec2::new(x + s * 0.86, y + s * 0.44),
        s * 0.12,
        color,
    );

    if cell_seed % 3 == 0 {
        append_notebook_ink_stroke(
            vertices,
            viewport,
            Vec2::new(x + s * 0.36, y + s * 0.18),
            Vec2::new(x + s * 0.58, y + s * 0.80),
            s * 0.09,
            color,
        );
    }
}

fn append_notebook_ink_stroke(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: ColorRgba,
) {
    let direction = Vec2::new(end.x - start.x, end.y - start.y);
    let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let normal = Vec2::new(-direction.y / length, direction.x / length);
    let half_width = width * 0.5;
    let offset = Vec2::new(normal.x * half_width, normal.y * half_width);
    let quad = [
        ndc_from_ui_screen(Vec2::new(start.x + offset.x, start.y + offset.y), viewport),
        ndc_from_ui_screen(Vec2::new(end.x + offset.x, end.y + offset.y), viewport),
        ndc_from_ui_screen(Vec2::new(end.x - offset.x, end.y - offset.y), viewport),
        ndc_from_ui_screen(Vec2::new(start.x - offset.x, start.y - offset.y), viewport),
    ];
    push_quad(vertices, quad[0], quad[1], quad[2], quad[3], color);
}

fn signed_unit_hash(mut value: u32) -> f32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
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

pub(crate) fn append_bitmap_font_screen_space_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    content: &str,
    rect: crate::ui_overlay::UiRect,
    font_size: f32,
    color: ColorRgba,
    anchor: UiTextAnchor,
    word_wrap: bool,
    fit_to_width: bool,
    texture_size: Vec2,
    prepared: &PreparedAsset,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let (effective_font_size, lines) =
        layout_ui_text_lines(content, rect.width, font_size, word_wrap, fit_to_width);
    let frame_count = metadata_u32_or(prepared, "atlas.frame_count", 3).max(1);
    let frame_width = metadata_f32_or(
        prepared,
        "atlas.frame_width",
        texture_size.x / frame_count as f32,
    )
    .max(1.0);
    let frame_height = metadata_f32_or(prepared, "atlas.frame_height", texture_size.y).max(1.0);
    let columns = metadata_u32_or(prepared, "atlas.columns", 10).max(1);
    let rows = metadata_u32_or(prepared, "atlas.rows", 10).max(1);
    let cell_width = metadata_f32_or(prepared, "atlas.cell_width", frame_width / columns as f32)
        .max(1.0);
    let cell_height = metadata_f32_or(prepared, "atlas.cell_height", frame_height / rows as f32)
        .max(1.0);
    let cell_inset = metadata_f32_or(prepared, "atlas.cell_inset", 0.0).max(0.0);
    let origin_x = metadata_f32_or(prepared, "atlas.origin_x", 0.0);
    let origin_y = metadata_f32_or(prepared, "atlas.origin_y", 0.0);
    let base_frame = bitmap_font_animation_frame(
        prepared,
        frame_count,
        metadata_u32_or(prepared, "atlas.active_variant", 0),
    );
    let atlas_line_height = metadata_f32_or(prepared, "atlas.line_height", 72.0).max(1.0);
    let advance_scale = effective_font_size / atlas_line_height;
    let glyph_height = effective_font_size * 1.06;
    let glyph_width = glyph_height * (cell_width / cell_height);
    let line_height = effective_font_size * 1.2;
    let line_widths = lines
        .iter()
        .map(|line| {
            line.chars()
                .map(|ch| bitmap_font_glyph_position(prepared, ch).advance * advance_scale)
                .sum::<f32>()
        })
        .collect::<Vec<_>>();
    let text_width = line_widths.iter().copied().fold(0.0, f32::max);
    let text_height = line_height * lines.len().max(1) as f32;

    let origin_x_screen = match anchor {
        UiTextAnchor::TopLeft => rect.x,
        UiTextAnchor::Center => rect.x + (rect.width - text_width).max(0.0) * 0.5,
    };
    let origin_y_screen = match anchor {
        UiTextAnchor::TopLeft => rect.y,
        UiTextAnchor::Center => rect.y + (rect.height - text_height).max(0.0) * 0.5,
    };

    for (line_index, line) in lines.iter().enumerate() {
        let line_origin_y = origin_y_screen + line_index as f32 * line_height;
        let line_origin_x = match anchor {
            UiTextAnchor::TopLeft => origin_x_screen,
            UiTextAnchor::Center => rect.x + (rect.width - line_widths[line_index]).max(0.0) * 0.5,
        };

        let mut cursor_x = 0.0;
        for (glyph_index, ch) in line.chars().enumerate() {
            let glyph = bitmap_font_glyph_position(prepared, ch);
            let glyph_advance = glyph.advance * advance_scale;
            let Some((column, row)) = glyph.position else {
                cursor_x += glyph_advance;
                continue;
            };
            let frame = bitmap_font_glyph_frame(prepared, frame_count, base_frame, glyph_index);
            let variant_origin_x = origin_x + frame.min(frame_count - 1) as f32 * frame_width;
            let glyph_origin_x = line_origin_x + cursor_x;
            let glyph_origin_y = line_origin_y;
            let min = Vec2::new(glyph_origin_x, glyph_origin_y);
            let max = Vec2::new(glyph_origin_x + glyph_width, glyph_origin_y + glyph_height);
            let (src_x0, src_y0, src_x1, src_y1) = bitmap_font_cell_rect(
                prepared,
                frame,
                column,
                row,
                variant_origin_x,
                origin_y,
                cell_width,
                cell_height,
                cell_inset,
            );
            let uv = TextureUvRect {
                u0: src_x0 / texture_size.x.max(1.0),
                v0: src_y0 / texture_size.y.max(1.0),
                u1: src_x1 / texture_size.x.max(1.0),
                v1: src_y1 / texture_size.y.max(1.0),
            };
            let bottom_left = ndc_from_ui_screen(Vec2::new(min.x, max.y), viewport);
            let bottom_right = ndc_from_ui_screen(max, viewport);
            let top_right = ndc_from_ui_screen(Vec2::new(max.x, min.y), viewport);
            let top_left = ndc_from_ui_screen(min, viewport);
            push_textured_quad(
                vertices,
                bottom_left,
                bottom_right,
                top_right,
                top_left,
                uv,
                color,
            );
            cursor_x += glyph_advance;
        }
    }
}

struct BitmapFontGlyphLookup {
    position: Option<(usize, usize)>,
    advance: f32,
}

fn bitmap_font_glyph_position(prepared: &PreparedAsset, ch: char) -> BitmapFontGlyphLookup {
    let default_advance = metadata_f32_or(prepared, "atlas.default_advance", 40.0);
    if ch == ' ' {
        return BitmapFontGlyphLookup {
            position: None,
            advance: metadata_f32_or(prepared, "atlas.space_advance", 28.0),
        };
    }
    if ch == '\t' {
        return BitmapFontGlyphLookup {
            position: None,
            advance: metadata_f32_or(prepared, "atlas.tab_advance", 112.0),
        };
    }

    let columns = metadata_u32_or(prepared, "atlas.columns", 10) as usize;
    let rows = metadata_u32_or(prepared, "atlas.rows", 10) as usize;
    for row in 0..rows {
        if let Some(glyphs) = prepared.metadata.get(&format!("glyphs.row_{row}")) {
            if let Some(column) = glyphs.chars().position(|glyph| glyph == ch) {
                if column < columns {
                    return BitmapFontGlyphLookup {
                        position: Some((column, row)),
                        advance: bitmap_font_glyph_advance(prepared, ch, default_advance),
                    };
                }
            }
        }
    }

    for row in 0..rows {
        if let Some(glyphs) = prepared.metadata.get(&format!("glyphs.row_{row}")) {
            if let Some(column) = glyphs.chars().position(|glyph| glyph == '?') {
                if column < columns {
                    return BitmapFontGlyphLookup {
                        position: Some((column, row)),
                        advance: bitmap_font_glyph_advance(prepared, ch, default_advance),
                    };
                }
            }
        }
    }

    BitmapFontGlyphLookup {
        position: None,
        advance: default_advance,
    }
}

fn bitmap_font_glyph_advance(prepared: &PreparedAsset, ch: char, default_advance: f32) -> f32 {
    for key in prepared.metadata.keys() {
        let Some(group) = key
            .strip_prefix("advance_groups.")
            .and_then(|rest| rest.strip_suffix(".chars"))
        else {
            continue;
        };
        if prepared
            .metadata
            .get(key)
            .map(|chars| chars.chars().any(|candidate| candidate == ch))
            .unwrap_or(false)
        {
            return metadata_f32_or(
                prepared,
                &format!("advance_groups.{group}.advance"),
                default_advance,
            );
        }
    }

    default_advance
}

fn bitmap_font_cell_rect(
    prepared: &PreparedAsset,
    frame: u32,
    column: usize,
    row: usize,
    fallback_origin_x: f32,
    fallback_origin_y: f32,
    fallback_cell_width: f32,
    fallback_cell_height: f32,
    inset: f32,
) -> (f32, f32, f32, f32) {
    if let (Some(x_lines), Some(y_lines)) =
        (bitmap_font_x_lines(prepared, frame), bitmap_font_y_lines(prepared))
    {
        let column_index = column.min(x_lines.len().saturating_sub(2));
        let row_index = row.min(y_lines.len().saturating_sub(2));
        let x0 = x_lines[column_index] + inset;
        let x1 = x_lines[column_index + 1] - inset;
        let y0 = y_lines[row_index] + inset;
        let y1 = y_lines[row_index + 1] - inset;
        return (x0, y0, x1, y1);
    }

    let x0 = fallback_origin_x + column as f32 * fallback_cell_width + inset;
    let y0 = fallback_origin_y + row as f32 * fallback_cell_height + inset;
    let x1 = x0 + fallback_cell_width - inset * 2.0;
    let y1 = y0 + fallback_cell_height - inset * 2.0;
    (x0, y0, x1, y1)
}

fn bitmap_font_x_lines(prepared: &PreparedAsset, frame: u32) -> Option<Vec<f32>> {
    let frame_width = metadata_f32_or(prepared, "atlas.frame_width", 0.0);
    let key = format!("calibration.x_lines_frame_{}", frame);
    let mut lines = parse_f32_list(prepared.metadata.get(&key)?)?;
    let frame_offset = frame as f32 * frame_width;
    for line in &mut lines {
        *line += frame_offset;
    }
    Some(lines)
}

fn bitmap_font_y_lines(prepared: &PreparedAsset) -> Option<Vec<f32>> {
    parse_f32_list(prepared.metadata.get("calibration.y_lines")?)
}

fn parse_f32_list(value: &str) -> Option<Vec<f32>> {
    let values = value
        .split(',')
        .map(|part| part.trim().parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() < 2 {
        None
    } else {
        Some(values)
    }
}

fn bitmap_font_animation_frame(
    prepared: &PreparedAsset,
    frame_count: u32,
    fallback_frame: u32,
) -> u32 {
    let frame_count = frame_count.max(1);
    let animated = prepared
        .metadata
        .get("animation.enabled")
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false);
    if !animated {
        return fallback_frame.min(frame_count - 1);
    }

    let fps = metadata_f32_or(prepared, "animation.fps", 6.0).max(0.1) as f64;
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    ((seconds * fps).floor() as u32) % frame_count
}

fn bitmap_font_glyph_frame(
    prepared: &PreparedAsset,
    frame_count: u32,
    base_frame: u32,
    glyph_index: usize,
) -> u32 {
    let frame_count = frame_count.max(1);
    let mode = prepared
        .metadata
        .get("animation.mode")
        .map(String::as_str)
        .unwrap_or("per_glyph_same_index");
    if mode == "per_glyph_offset" {
        return (base_frame + glyph_index as u32) % frame_count;
    }
    base_frame % frame_count
}

fn metadata_f32_or(prepared: &PreparedAsset, key: &str, fallback: f32) -> f32 {
    prepared
        .metadata
        .get(key)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(fallback)
}

fn metadata_u32_or(prepared: &PreparedAsset, key: &str, fallback: u32) -> u32 {
    prepared
        .metadata
        .get(key)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback)
}
