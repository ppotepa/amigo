use amigo_math::ColorRgba;
use crate::ui_overlay::{
    append_color_picker_rgb_primitives,
    append_curve_editor_primitives, append_dropdown_header_primitives,
    append_dropdown_popup_primitives, append_option_set_primitives, append_slider_primitives,
    append_tab_view_header_primitives, append_toggle_primitives, UiDrawPrimitive, UiLayoutNode,
    UiOverlayNodeKind, UiRect, UiTextAnchor,
};
use crate::ui_overlay::layout::group_box_label_height;

pub(crate) fn append_layout_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
) {
    if let Some(background) = layout.node.style.background {
        primitives.push(UiDrawPrimitive::Quad {
            rect: layout.rect,
            color: background,
        });
    }

    if let Some(border_color) = layout.node.style.border_color {
        append_border_primitives(
            primitives,
            layout.rect,
            border_color,
            layout.node.style.border_width.max(0.0),
        );
    }

    match &layout.node.kind {
        UiOverlayNodeKind::Text { content, font } => primitives.push(UiDrawPrimitive::Text {
            rect: layout.rect,
            content: content.clone(),
            color: layout.node.style.color.unwrap_or(ColorRgba::WHITE),
            font_size: layout.node.style.font_size.max(8.0),
            font: font.clone(),
            anchor: layout.node.style.text_anchor,
            word_wrap: layout.node.style.word_wrap,
            fit_to_width: layout.node.style.fit_to_width,
        }),
        UiOverlayNodeKind::Button { text, font } => {
            if layout.node.style.background.is_none() {
                primitives.push(UiDrawPrimitive::Quad {
                    rect: layout.rect,
                    color: ColorRgba::new(0.2, 0.33, 0.66, 1.0),
                });
            }
            primitives.push(UiDrawPrimitive::Text {
                rect: layout
                    .rect
                    .inset(layout.node.style.padding.max(0.0).max(8.0)),
                content: text.clone(),
                color: layout.node.style.color.unwrap_or(ColorRgba::WHITE),
                font_size: layout.node.style.font_size.max(14.0),
                font: font.clone(),
                anchor: UiTextAnchor::Center,
                word_wrap: layout.node.style.word_wrap,
                fit_to_width: layout.node.style.fit_to_width,
            });
        }
        UiOverlayNodeKind::GroupBox { label, font } => {
            primitives.push(UiDrawPrimitive::Text {
                rect: UiRect::new(
                    layout.rect.x + layout.node.style.padding.max(0.0),
                    layout.rect.y,
                    (layout.rect.width - layout.node.style.padding.max(0.0) * 2.0).max(0.0),
                    group_box_label_height(&layout.node),
                ),
                content: label.clone(),
                color: layout.node.style.color.unwrap_or(ColorRgba::WHITE),
                font_size: layout.node.style.font_size.max(8.0),
                font: font.clone(),
                anchor: UiTextAnchor::TopLeft,
                word_wrap: false,
                fit_to_width: true,
            });
        }
        UiOverlayNodeKind::ProgressBar { value } => primitives.push(UiDrawPrimitive::ProgressBar {
            rect: layout.rect,
            value: value.clamp(0.0, 1.0),
            background: layout
                .node
                .style
                .background
                .unwrap_or(ColorRgba::new(0.19, 0.21, 0.29, 1.0)),
            foreground: layout
                .node
                .style
                .color
                .unwrap_or(ColorRgba::new(0.4, 0.8, 0.53, 1.0)),
        }),
        UiOverlayNodeKind::Slider { value, .. } => {
            append_slider_primitives(layout, primitives, value.clamp(0.0, 1.0));
        }
        UiOverlayNodeKind::Toggle {
            checked,
            text,
            font,
        } => {
            append_toggle_primitives(layout, primitives, *checked, text, font);
        }
        UiOverlayNodeKind::OptionSet {
            selected,
            options,
            font,
        } => append_option_set_primitives(layout, primitives, selected, options, font),
        UiOverlayNodeKind::Dropdown { selected, font, .. } => {
            append_dropdown_header_primitives(layout, primitives, selected, font)
        }
        UiOverlayNodeKind::TabView {
            selected,
            tabs,
            font,
        } => append_tab_view_header_primitives(layout, primitives, selected, tabs, font),
        UiOverlayNodeKind::ColorPickerRgb { color } => {
            append_color_picker_rgb_primitives(layout, primitives, *color);
        }
        UiOverlayNodeKind::CurveEditor { points } => {
            append_curve_editor_primitives(layout, primitives, points);
        }
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::Spacer => {}
    }

    for child in &layout.children {
        append_layout_primitives(child, primitives);
    }
}

pub(crate) fn append_layout_popup_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
) {
    for child in &layout.children {
        append_layout_popup_primitives(child, primitives);
    }

    if let UiOverlayNodeKind::Dropdown {
        selected,
        options,
        expanded: true,
        scroll_offset,
        font,
    } = &layout.node.kind
    {
        append_dropdown_popup_primitives(
            layout,
            primitives,
            selected,
            options,
            *scroll_offset,
            font,
        );
    }
}

pub(crate) fn append_border_primitives(
    primitives: &mut Vec<UiDrawPrimitive>,
    rect: UiRect,
    color: ColorRgba,
    width: f32,
) {
    if width <= 0.0 {
        return;
    }

    let horizontal = width.min(rect.height * 0.5);
    let vertical = width.min(rect.width * 0.5);
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(rect.x, rect.y, rect.width, horizontal),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(
            rect.x,
            rect.y + rect.height - horizontal,
            rect.width,
            horizontal,
        ),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(rect.x, rect.y, vertical, rect.height),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(
            rect.x + rect.width - vertical,
            rect.y,
            vertical,
            rect.height,
        ),
        color,
    });
}

