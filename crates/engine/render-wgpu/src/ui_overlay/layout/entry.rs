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


