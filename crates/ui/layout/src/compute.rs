use crate::flow::layout_node;
use crate::measure::measure_element;
use crate::model::{LayoutElement, LayoutNode, LayoutRect, LayoutViewport, LayoutViewportScaling};
use crate::viewport::transform_layout_for_viewport;

pub fn compute_layout<T: Clone>(
    root_path: &str,
    viewport: LayoutViewport,
    root: &LayoutElement<T>,
    document_viewport: Option<(LayoutViewport, LayoutViewportScaling)>,
) -> LayoutNode<T> {
    let layout_viewport = match document_viewport {
        Some((vp, LayoutViewportScaling::Fixed | LayoutViewportScaling::Fit)) => {
            LayoutViewport::new(vp.width.max(1.0), vp.height.max(1.0))
        }
        _ => viewport,
    };

    let measured = measure_element(root);
    let width = root.style.width.unwrap_or(measured.0).max(0.0);
    let height = root.style.height.unwrap_or(measured.1).max(0.0);
    let x = resolve_screen_axis(
        root.style.left,
        root.style.right,
        layout_viewport.width,
        width,
    );
    let y = resolve_screen_axis(
        root.style.top,
        root.style.bottom,
        layout_viewport.height,
        height,
    );
    let root_rect = LayoutRect::new(x, y, width, height);
    let layout = layout_node(
        root_path,
        root,
        root_rect,
        root.id.clone().unwrap_or_else(|| "root".to_owned()),
        0,
    );
    transform_layout_for_viewport(layout, viewport, document_viewport)
}

pub(crate) fn resolve_screen_axis(
    start: Option<f32>,
    end: Option<f32>,
    extent: f32,
    size: f32,
) -> f32 {
    match (start, end) {
        (Some(start), _) => start,
        (None, Some(end)) => (extent - end - size).max(0.0),
        (None, None) => 0.0,
    }
}
