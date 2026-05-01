pub fn build_ui_overlay_primitives(
    viewport: UiViewportSize,
    documents: &[UiOverlayDocument],
) -> Vec<UiDrawPrimitive> {
    let mut ordered = documents.to_vec();
    ordered.sort_by_key(|document| document.layer);

    let mut primitives = Vec::new();
    let mut popup_primitives = Vec::new();
    for document in &ordered {
        let layout = build_ui_layout_tree(viewport, document);
        append_layout_primitives(&layout, &mut primitives);
        append_layout_popup_primitives(&layout, &mut popup_primitives);
    }
    primitives.extend(popup_primitives);

    primitives
}

pub fn build_ui_layout_tree(
    viewport: UiViewportSize,
    document: &UiOverlayDocument,
) -> UiLayoutNode {
    let layout_viewport = match document.viewport {
        Some(UiOverlayViewport {
            width,
            height,
            scaling: UiOverlayViewportScaling::Fixed | UiOverlayViewportScaling::Fit,
        }) => UiViewportSize::new(width.max(1.0), height.max(1.0)),
        Some(UiOverlayViewport {
            scaling: UiOverlayViewportScaling::Expand,
            ..
        })
        | None => viewport,
    };
    let measured = measure_node(&document.root);
    let width = document.root.style.width.unwrap_or(measured.x).max(0.0);
    let height = document.root.style.height.unwrap_or(measured.y).max(0.0);
    let x = resolve_screen_axis(
        document.root.style.left,
        document.root.style.right,
        layout_viewport.width,
        width,
    );
    let y = resolve_screen_axis(
        document.root.style.top,
        document.root.style.bottom,
        layout_viewport.height,
        height,
    );
    let root_rect = UiRect::new(x, y, width, height);
    let layout = layout_node(
        &document.entity_name,
        &document.root,
        root_rect,
        document
            .root
            .id
            .clone()
            .unwrap_or_else(|| "root".to_owned()),
        0,
    );

    transform_layout_for_viewport(layout, viewport, document.viewport)
}

