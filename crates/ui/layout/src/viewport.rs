use crate::model::{LayoutNode, LayoutRect, LayoutViewport, LayoutViewportScaling};

pub(crate) fn transform_layout_for_viewport<T: Clone>(
    layout: LayoutNode<T>,
    viewport: LayoutViewport,
    document_viewport: Option<(LayoutViewport, LayoutViewportScaling)>,
) -> LayoutNode<T> {
    let Some((document_viewport, scaling_mode)) = document_viewport else {
        return layout;
    };

    if scaling_mode == LayoutViewportScaling::Expand {
        return layout;
    }

    let design_width = document_viewport.width.max(1.0);
    let design_height = document_viewport.height.max(1.0);
    let scale = match scaling_mode {
        LayoutViewportScaling::Expand => 1.0,
        LayoutViewportScaling::Fixed => 1.0,
        LayoutViewportScaling::Fit => {
            (viewport.width / design_width).min(viewport.height / design_height)
        }
    }
    .max(0.0);

    let offset_x = (viewport.width - design_width * scale) * 0.5;
    let offset_y = (viewport.height - design_height * scale) * 0.5;

    transform_layout_node(layout, offset_x, offset_y, scale)
}

fn transform_layout_node<T: Clone>(
    node: LayoutNode<T>,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
) -> LayoutNode<T> {
    LayoutNode {
        path: node.path,
        rect: LayoutRect::new(
            offset_x + node.rect.x * scale,
            offset_y + node.rect.y * scale,
            node.rect.width * scale,
            node.rect.height * scale,
        ),
        data: node.data,
        children: node
            .children
            .into_iter()
            .map(|child| transform_layout_node(child, offset_x, offset_y, scale))
            .collect(),
    }
}
