pub(crate) fn hit_test_ui_layout(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if let Some(path) = hit_test_expanded_dropdown(node, x, y) {
        return Some(path);
    }

    hit_test_ui_layout_normal(node, x, y)
}

fn hit_test_ui_layout_normal(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if x < node.rect.x
        || y < node.rect.y
        || x > node.rect.x + node.rect.width
        || y > node.rect.y + node.rect.height
    {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_ui_layout_normal(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}

fn hit_test_expanded_dropdown(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_expanded_dropdown(child, x, y) {
            return Some(path);
        }
    }

    let UiOverlayNodeKind::Dropdown {
        expanded: true,
        options,
        scroll_offset: _,
        ..
    } = &node.node.kind
    else {
        return None;
    };

    let row_height = 38.0_f32.min(node.rect.height.max(0.0));
    let total_height = row_height * (dropdown_visible_option_count(options.len()) as f32 + 1.0);
    if x >= node.rect.x
        && x <= node.rect.x + node.rect.width
        && y >= node.rect.y
        && y <= node.rect.y + total_height
    {
        return Some(node.path.clone());
    }

    None
}

pub(crate) fn dropdown_visible_option_count(option_count: usize) -> usize {
    option_count.min(10)
}

pub(crate) fn find_ui_layout_node<'a>(
    node: &'a OverlayUiLayoutNode,
    path: &str,
) -> Option<&'a OverlayUiLayoutNode> {
    if node.path == path {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_ui_layout_node(child, path) {
            return Some(found);
        }
    }
    None
}
