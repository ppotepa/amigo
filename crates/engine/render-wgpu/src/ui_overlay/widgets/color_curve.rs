pub(crate) fn append_color_picker_rgb_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    color: ColorRgba,
) {
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.12, 0.14, 0.2, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.88, 0.94, 1.0, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));

    primitives.push(UiDrawPrimitive::Quad {
        rect: layout.rect,
        color: background,
    });
    append_border_primitives(primitives, layout.rect, border, 1.0);

    let padding = 8.0;
    let swatch = UiRect::new(
        layout.rect.x + padding,
        layout.rect.y + padding,
        54.0_f32.min((layout.rect.width - padding * 2.0).max(0.0)),
        (layout.rect.height - padding * 2.0).max(0.0),
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: swatch,
        color,
    });
    append_border_primitives(primitives, swatch, border, 1.0);

    let slider_x = swatch.x + swatch.width + 10.0;
    let slider_width = (layout.rect.x + layout.rect.width - padding - slider_x).max(0.0);
    let slider_height = 22.0;
    for (index, (label, value, channel_color)) in [
        ("R", color.r, ColorRgba::new(0.95, 0.24, 0.28, 1.0)),
        ("G", color.g, ColorRgba::new(0.32, 0.86, 0.42, 1.0)),
        ("B", color.b, ColorRgba::new(0.26, 0.54, 1.0, 1.0)),
    ]
    .into_iter()
    .enumerate()
    {
        let y = layout.rect.y + padding + index as f32 * (slider_height + 10.0);
        let label_rect = UiRect::new(slider_x, y, 18.0, slider_height);
        primitives.push(UiDrawPrimitive::Text {
            rect: label_rect,
            content: label.to_owned(),
            color: foreground,
            font_size: layout.node.style.font_size.max(12.0),
            font: None,
            anchor: UiTextAnchor::Center,
            word_wrap: false,
            fit_to_width: true,
        });
        let track = UiRect::new(
            slider_x + 24.0,
            y + 5.0,
            (slider_width - 24.0).max(0.0),
            12.0,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect: track,
            color: ColorRgba::new(0.04, 0.05, 0.08, 1.0),
        });
        primitives.push(UiDrawPrimitive::Quad {
            rect: UiRect::new(
                track.x,
                track.y,
                track.width * value.clamp(0.0, 1.0),
                track.height,
            ),
            color: channel_color,
        });
    }
}

pub(crate) fn append_curve_editor_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    points: &[UiOverlayCurvePoint],
) {
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.12, 0.14, 0.2, 1.0));
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
    primitives.push(UiDrawPrimitive::Quad {
        rect: layout.rect,
        color: background,
    });
    append_border_primitives(primitives, layout.rect, border, 1.0);

    let plot = layout.rect.inset(10.0);
    if plot.width <= 0.0 || plot.height <= 0.0 {
        return;
    }
    for index in 1..4 {
        let x = plot.x + plot.width * index as f32 / 4.0;
        primitives.push(UiDrawPrimitive::Quad {
            rect: UiRect::new(x, plot.y, 1.0, plot.height),
            color: ColorRgba::new(border.r, border.g, border.b, border.a * 0.45),
        });
    }
    for index in 1..4 {
        let y = plot.y + plot.height * index as f32 / 4.0;
        primitives.push(UiDrawPrimitive::Quad {
            rect: UiRect::new(plot.x, y, plot.width, 1.0),
            color: ColorRgba::new(border.r, border.g, border.b, border.a * 0.45),
        });
    }

    let points: Vec<(f32, f32)> = normalized_curve_points(points)
        .into_iter()
        .map(|point| {
            let x = plot.x + plot.width * point.t.clamp(0.0, 1.0);
            let y = plot.y + plot.height * (1.0 - point.value.clamp(0.0, 1.0));
            (x, y)
        })
        .collect();
    for pair in points.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        let rect = UiRect::new(
            x0.min(x1),
            ((y0 + y1) * 0.5 - 1.5).clamp(plot.y, plot.y + plot.height),
            (x1 - x0).abs().max(1.0),
            3.0,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect,
            color: foreground,
        });
    }
    for (x, y) in points {
        primitives.push(UiDrawPrimitive::Quad {
            rect: UiRect::new(x - 4.0, y - 4.0, 8.0, 8.0),
            color: foreground,
        });
    }
}
