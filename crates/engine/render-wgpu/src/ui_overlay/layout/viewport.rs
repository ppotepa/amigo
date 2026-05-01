fn transform_layout_for_viewport(
    layout: UiLayoutNode,
    viewport: UiViewportSize,
    document_viewport: Option<UiOverlayViewport>,
) -> UiLayoutNode {
    let Some(document_viewport) = document_viewport else {
        return layout;
    };

    if document_viewport.scaling == UiOverlayViewportScaling::Expand {
        return layout;
    }

    let design_width = document_viewport.width.max(1.0);
    let design_height = document_viewport.height.max(1.0);
    let scale = match document_viewport.scaling {
        UiOverlayViewportScaling::Expand => 1.0,
        UiOverlayViewportScaling::Fixed => 1.0,
        UiOverlayViewportScaling::Fit => {
            (viewport.width / design_width).min(viewport.height / design_height)
        }
    }
    .max(0.0);
    let offset = Vec2::new(
        (viewport.width - design_width * scale) * 0.5,
        (viewport.height - design_height * scale) * 0.5,
    );

    transform_layout_node(layout, offset, scale)
}

fn transform_layout_node(node: UiLayoutNode, offset: Vec2, scale: f32) -> UiLayoutNode {
    UiLayoutNode {
        path: node.path,
        rect: UiRect::new(
            offset.x + node.rect.x * scale,
            offset.y + node.rect.y * scale,
            node.rect.width * scale,
            node.rect.height * scale,
        ),
        node: UiOverlayNode {
            id: node.node.id,
            kind: node.node.kind,
            style: scale_overlay_style(node.node.style, scale),
            children: node.node.children,
        },
        children: node
            .children
            .into_iter()
            .map(|child| transform_layout_node(child, offset, scale))
            .collect(),
    }
}

fn scale_overlay_style(mut style: UiOverlayStyle, scale: f32) -> UiOverlayStyle {
    style.padding *= scale;
    style.gap *= scale;
    style.border_width *= scale;
    style.border_radius *= scale;
    style.font_size *= scale;
    style
}

