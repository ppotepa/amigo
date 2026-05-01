pub(crate) fn append_dropdown_popup_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    options: &[String],
    scroll_offset: f32,
    font: &Option<AssetKey>,
) {
    let row_height = 38.0_f32.min(layout.rect.height.max(0.0));
    if row_height <= 0.0 {
        return;
    }
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));
    let visible_count = dropdown_visible_option_count(options.len());
    let max_offset = options.len().saturating_sub(visible_count) as f32;
    let scroll_offset = scroll_offset.clamp(0.0, max_offset);
    let first_index = scroll_offset.floor() as usize;
    let fractional_offset = scroll_offset - first_index as f32;
    let popup_rect = UiRect::new(
        layout.rect.x,
        layout.rect.y + row_height,
        layout.rect.width,
        row_height * visible_count as f32,
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: popup_rect,
        color: background,
    });
    append_border_primitives(primitives, popup_rect, border, 1.0);

    let needs_scrollbar = options.len() > visible_count;
    let scrollbar_width = if needs_scrollbar { 10.0 } else { 0.0 };
    let option_width = (layout.rect.width - scrollbar_width).max(0.0);
    let render_count = (visible_count + 1).min(options.len().saturating_sub(first_index));
    for visible_index in 0..render_count {
        let option_index = first_index + visible_index;
        let Some(option) = options.get(option_index) else {
            continue;
        };
        let rect = UiRect::new(
            layout.rect.x,
            layout.rect.y + row_height * (visible_index as f32 + 1.0 - fractional_offset),
            option_width,
            row_height,
        );
        if rect.y + rect.height <= popup_rect.y || rect.y >= popup_rect.y + popup_rect.height {
            continue;
        }
        primitives.push(UiDrawPrimitive::Quad {
            rect,
            color: if option == selected {
                foreground
            } else {
                background
            },
        });
        append_border_primitives(primitives, rect, border, 1.0);
        primitives.push(UiDrawPrimitive::Text {
            rect: rect.inset(8.0),
            content: option.clone(),
            color: if option == selected {
                background
            } else {
                foreground
            },
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::TopLeft,
            word_wrap: false,
            fit_to_width: true,
        });
    }
    if needs_scrollbar {
        append_dropdown_scrollbar_primitives(
            primitives,
            popup_rect,
            border,
            foreground,
            options.len(),
            visible_count,
            scroll_offset,
        );
    }
}

fn dropdown_visible_option_count(option_count: usize) -> usize {
    option_count.min(10)
}

fn append_dropdown_scrollbar_primitives(
    primitives: &mut Vec<UiDrawPrimitive>,
    popup_rect: UiRect,
    track_color: ColorRgba,
    thumb_color: ColorRgba,
    option_count: usize,
    visible_count: usize,
    scroll_offset: f32,
) {
    let track_width = 10.0_f32.min(popup_rect.width.max(0.0));
    if track_width <= f32::EPSILON || option_count <= visible_count || visible_count == 0 {
        return;
    }
    let track = UiRect::new(
        popup_rect.x + popup_rect.width - track_width,
        popup_rect.y,
        track_width,
        popup_rect.height,
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: track,
        color: ColorRgba::new(
            track_color.r,
            track_color.g,
            track_color.b,
            track_color.a * 0.55,
        ),
    });

    let max_offset = option_count.saturating_sub(visible_count) as f32;
    let visible_ratio = (visible_count as f32 / option_count as f32).clamp(0.05, 1.0);
    let thumb_height = (track.height * visible_ratio).clamp(18.0, track.height);
    let travel = (track.height - thumb_height).max(0.0);
    let offset_ratio = if max_offset <= f32::EPSILON {
        0.0
    } else {
        (scroll_offset / max_offset).clamp(0.0, 1.0)
    };
    let thumb = UiRect::new(
        track.x + 2.0,
        track.y + travel * offset_ratio + 2.0,
        (track.width - 4.0).max(0.0),
        (thumb_height - 4.0).max(0.0),
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: thumb,
        color: thumb_color,
    });
}

