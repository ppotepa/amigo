pub(crate) fn append_tab_view_header_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    tabs: &[UiOverlayTab],
    font: &Option<AssetKey>,
) {
    if tabs.is_empty() {
        return;
    }
    let header_height = crate::ui_overlay::layout::tab_view_header_height(&layout.node);
    let tab_width = layout.rect.width / tabs.len() as f32;
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout.node.style.color.unwrap_or(ColorRgba::WHITE);
    let active = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    for (index, tab) in tabs.iter().enumerate() {
        let rect = UiRect::new(
            layout.rect.x + index as f32 * tab_width,
            layout.rect.y,
            tab_width,
            header_height,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect,
            color: if tab.id == selected {
                active
            } else {
                background
            },
        });
        primitives.push(UiDrawPrimitive::Text {
            rect: rect.inset(6.0),
            content: tab.label.clone(),
            color: foreground,
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::Center,
            word_wrap: false,
            fit_to_width: true,
        });
    }
}

