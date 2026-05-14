pub fn tab_view_tab_from_mouse(
    rect: UiRect,
    node: &UiOverlayNode,
    tabs: &[UiOverlayTab],
    mouse_x: f32,
    mouse_y: f32,
) -> Option<String> {
    if tabs.is_empty() || mouse_y < rect.y || mouse_y > rect.y + tab_view_header_height(node) {
        return None;
    }
    let tab_width = rect.width / tabs.len() as f32;
    if tab_width <= f32::EPSILON || mouse_x < rect.x || mouse_x > rect.x + rect.width {
        return None;
    }
    let index =
        (((mouse_x - rect.x) / tab_width).clamp(0.0, 0.999_999) * tabs.len() as f32).floor()
            as usize;
    tabs.get(index).map(|tab| tab.id.clone())
}

pub(crate) fn tab_view_header_height(node: &UiOverlayNode) -> f32 {
    (node.style.font_size.max(14.0) * 1.2 + node.style.padding.max(0.0) * 2.0).max(38.0)
}

pub(crate) fn group_box_label_height(node: &UiOverlayNode) -> f32 {
    node.style.font_size.max(8.0) * 1.2
}
