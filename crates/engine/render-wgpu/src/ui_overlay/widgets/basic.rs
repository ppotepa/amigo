use amigo_assets::AssetKey;
use amigo_math::ColorRgba;
use crate::ui_overlay::{
    append_border_primitives,
    helpers::normalized_curve_points,
    UiDrawPrimitive,
    UiLayoutNode,
    UiOverlayCurvePoint,
    UiOverlayTab,
    UiRect,
    UiTextAnchor,
};

pub(crate) fn append_slider_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    value: f32,
) {
    let track = layout.rect.inset(4.0);
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
    primitives.push(UiDrawPrimitive::Quad {
        rect: track,
        color: background,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(track.x, track.y, track.width * value, track.height),
        color: foreground,
    });
    let thumb_width = 10.0_f32.min(layout.rect.width.max(0.0));
    let thumb_x = (track.x + track.width * value - thumb_width * 0.5)
        .max(layout.rect.x)
        .min(layout.rect.x + layout.rect.width - thumb_width);
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(thumb_x, layout.rect.y, thumb_width, layout.rect.height),
        color: foreground,
    });
}

pub(crate) fn append_toggle_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    checked: bool,
    text: &str,
    font: &Option<AssetKey>,
) {
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.9, 0.94, 1.0, 1.0));
    let accent = if checked {
        foreground
    } else {
        layout
            .node
            .style
            .border_color
            .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0))
    };
    let switch_width = 42.0_f32.min(layout.rect.width.max(0.0));
    let switch_rect = UiRect::new(
        layout.rect.x,
        layout.rect.y,
        switch_width,
        layout.rect.height,
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: switch_rect.inset(5.0),
        color: layout
            .node
            .style
            .background
            .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0)),
    });
    let knob = if checked {
        UiRect::new(
            switch_rect.x + switch_rect.width - 18.0,
            switch_rect.y + 8.0,
            12.0,
            (switch_rect.height - 16.0).max(0.0),
        )
    } else {
        UiRect::new(
            switch_rect.x + 6.0,
            switch_rect.y + 8.0,
            12.0,
            (switch_rect.height - 16.0).max(0.0),
        )
    };
    primitives.push(UiDrawPrimitive::Quad {
        rect: knob,
        color: accent,
    });
    if !text.is_empty() {
        primitives.push(UiDrawPrimitive::Text {
            rect: UiRect::new(
                layout.rect.x + switch_width + 8.0,
                layout.rect.y,
                (layout.rect.width - switch_width - 8.0).max(0.0),
                layout.rect.height,
            ),
            content: text.to_owned(),
            color: foreground,
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::TopLeft,
            word_wrap: layout.node.style.word_wrap,
            fit_to_width: layout.node.style.fit_to_width,
        });
    }
}

pub(crate) fn append_option_set_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    options: &[String],
    font: &Option<AssetKey>,
) {
    if options.is_empty() {
        return;
    }
    let segment_width = layout.rect.width / options.len() as f32;
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
    for (index, option) in options.iter().enumerate() {
        let rect = UiRect::new(
            layout.rect.x + index as f32 * segment_width,
            layout.rect.y,
            segment_width,
            layout.rect.height,
        );
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
            rect: rect.inset(6.0),
            content: option.clone(),
            color: if option == selected {
                background
            } else {
                foreground
            },
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::Center,
            word_wrap: false,
            fit_to_width: true,
        });
    }
}

pub(crate) fn append_dropdown_header_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    font: &Option<AssetKey>,
) {
    let row_height = 38.0_f32.min(layout.rect.height.max(0.0));
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
    let header = UiRect::new(layout.rect.x, layout.rect.y, layout.rect.width, row_height);
    primitives.push(UiDrawPrimitive::Quad {
        rect: header,
        color: background,
    });
    append_border_primitives(primitives, header, border, 1.0);
    primitives.push(UiDrawPrimitive::Text {
        rect: header.inset(8.0),
        content: format!("{selected} v"),
        color: foreground,
        font_size: layout.node.style.font_size.max(14.0),
        font: font.clone(),
        anchor: UiTextAnchor::TopLeft,
        word_wrap: false,
        fit_to_width: true,
    });
}
