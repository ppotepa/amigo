use crate::ui_overlay::layout::group_box_label_height;
use crate::ui_overlay::{
    UiDrawPrimitive, UiLayoutNode, UiOverlayNodeKind, UiOverlayStyle, UiRect, UiTextAnchor,
    append_color_picker_rgb_primitives, append_curve_editor_primitives,
    append_dropdown_header_primitives, append_dropdown_popup_primitives,
    append_option_set_primitives, append_slider_primitives, append_tab_view_header_primitives,
    append_toggle_primitives,
};
use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

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
        UiOverlayNodeKind::Text { content, font } => push_text_with_effects(
            primitives,
            layout.rect,
            content.clone(),
            font.clone(),
            &layout.node.style,
            layout.node.style.text_anchor,
            layout.node.style.font_size.max(8.0),
            layout.node.style.word_wrap,
            layout.node.style.fit_to_width,
        ),
        UiOverlayNodeKind::Button { text, font } => {
            if layout.node.style.background.is_none() {
                primitives.push(UiDrawPrimitive::Quad {
                    rect: layout.rect,
                    color: ColorRgba::new(0.2, 0.33, 0.66, 1.0),
                });
            }
            push_text_with_effects(
                primitives,
                layout
                    .rect
                    .inset(layout.node.style.padding.max(0.0).max(8.0)),
                text.clone(),
                font.clone(),
                &layout.node.style,
                UiTextAnchor::Center,
                layout.node.style.font_size.max(14.0),
                layout.node.style.word_wrap,
                layout.node.style.fit_to_width,
            );
        }
        UiOverlayNodeKind::GroupBox { label, font } => {
            push_text_with_effects(
                primitives,
                UiRect::new(
                    layout.rect.x + layout.node.style.padding.max(0.0),
                    layout.rect.y,
                    (layout.rect.width - layout.node.style.padding.max(0.0) * 2.0).max(0.0),
                    group_box_label_height(&layout.node),
                ),
                label.clone(),
                font.clone(),
                &layout.node.style,
                UiTextAnchor::TopLeft,
                layout.node.style.font_size.max(8.0),
                false,
                true,
            );
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

fn push_text_with_effects(
    primitives: &mut Vec<UiDrawPrimitive>,
    rect: UiRect,
    content: String,
    font: Option<AssetKey>,
    style: &UiOverlayStyle,
    anchor: UiTextAnchor,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) {
    if let Some(glow) = style.text_glow {
        let passes = glow.passes.max(1);
        let step = glow.radius.max(0.0) / passes as f32;

        for pass in 1..=passes {
            let radius = pass as f32 * step;
            let alpha = glow.intensity.max(0.0) / pass as f32;

            for (dx, dy) in [
                (-radius, 0.0),
                (radius, 0.0),
                (0.0, -radius),
                (0.0, radius),
                (-radius * 0.7, -radius * 0.7),
                (radius * 0.7, -radius * 0.7),
                (-radius * 0.7, radius * 0.7),
                (radius * 0.7, radius * 0.7),
            ] {
                push_text_primitive(
                    primitives,
                    offset_rect(rect, dx, dy),
                    content.clone(),
                    color_with_alpha_mul(glow.color, alpha),
                    font_size,
                    font.clone(),
                    anchor,
                    word_wrap,
                    fit_to_width,
                );
            }
        }
    }

    if let Some(outline) = style.text_outline {
        let width = outline.width.max(0.0);

        if width > 0.0 {
            for (dx, dy) in [
                (-width, 0.0),
                (width, 0.0),
                (0.0, -width),
                (0.0, width),
                (-width, -width),
                (width, -width),
                (-width, width),
                (width, width),
            ] {
                push_text_primitive(
                    primitives,
                    offset_rect(rect, dx, dy),
                    content.clone(),
                    outline.color,
                    font_size,
                    font.clone(),
                    anchor,
                    word_wrap,
                    fit_to_width,
                );
            }
        }
    }

    if let Some(shadow) = style.text_shadow {
        push_text_primitive(
            primitives,
            offset_rect(rect, shadow.offset.x, shadow.offset.y),
            content.clone(),
            shadow.color,
            font_size,
            font.clone(),
            anchor,
            word_wrap,
            fit_to_width,
        );
    }

    push_text_primitive(
        primitives,
        rect,
        content,
        style.color.unwrap_or(ColorRgba::WHITE),
        font_size,
        font,
        anchor,
        word_wrap,
        fit_to_width,
    );
}

fn push_text_primitive(
    primitives: &mut Vec<UiDrawPrimitive>,
    rect: UiRect,
    content: String,
    color: ColorRgba,
    font_size: f32,
    font: Option<AssetKey>,
    anchor: UiTextAnchor,
    word_wrap: bool,
    fit_to_width: bool,
) {
    primitives.push(UiDrawPrimitive::Text {
        rect,
        content,
        color,
        font_size,
        font,
        anchor,
        word_wrap,
        fit_to_width,
    });
}

fn offset_rect(rect: UiRect, dx: f32, dy: f32) -> UiRect {
    UiRect {
        x: rect.x + dx,
        y: rect.y + dy,
        width: rect.width,
        height: rect.height,
    }
}

fn color_with_alpha_mul(color: ColorRgba, alpha: f32) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
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
