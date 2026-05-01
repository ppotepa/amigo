fn layout_node(node: &UiNode, path: String, available: UiRect, index: usize) -> UiLayoutNode {
    let rect = resolve_rect(node, available, index);
    let children = match &node.kind {
        UiNodeKind::Column => layout_column(node, &path, rect),
        UiNodeKind::Row => layout_row(node, &path, rect),
        UiNodeKind::GroupBox { .. } => layout_group_box(node, &path, rect),
        UiNodeKind::TabView { selected, tabs, .. } => {
            layout_tab_view(node, &path, rect, selected, tabs)
        }
        UiNodeKind::Stack | UiNodeKind::Panel => layout_stack(node, &path, rect),
        UiNodeKind::Text { .. }
        | UiNodeKind::Button { .. }
        | UiNodeKind::ProgressBar { .. }
        | UiNodeKind::Slider { .. }
        | UiNodeKind::Toggle { .. }
        | UiNodeKind::OptionSet { .. }
        | UiNodeKind::Dropdown { .. }
        | UiNodeKind::ColorPickerRgb { .. }
        | UiNodeKind::CurveEditor { .. }
        | UiNodeKind::Spacer => Vec::new(),
    };

    UiLayoutNode {
        path,
        rect,
        node: node.clone(),
        children,
    }
}

fn layout_group_box(node: &UiNode, parent_path: &str, rect: UiRect) -> Vec<UiLayoutNode> {
    let label_height = group_box_label_height(node);
    let content = UiRect::new(
        rect.x + node.style.padding,
        rect.y + label_height + node.style.padding,
        (rect.width - node.style.padding * 2.0).max(0.0),
        (rect.height - label_height - node.style.padding * 2.0).max(0.0),
    );
    layout_column_contents(node, parent_path, content)
}

fn layout_column(node: &UiNode, parent_path: &str, rect: UiRect) -> Vec<UiLayoutNode> {
    let content = rect.inset(node.style.padding);
    layout_column_contents(node, parent_path, content)
}

fn layout_column_contents(node: &UiNode, parent_path: &str, content: UiRect) -> Vec<UiLayoutNode> {
    let mut cursor_y = content.y;
    let mut children = Vec::with_capacity(node.children.len());

    for (index, child) in node.children.iter().enumerate() {
        let child_rect = UiRect::new(content.x, cursor_y, content.width, content.height);
        let path = child_path(parent_path, child, index);
        let layout = layout_node(child, path, child_rect, index);
        cursor_y = layout.rect.y + layout.rect.height + node.style.gap;
        children.push(layout);
    }

    children
}

fn layout_tab_view(
    node: &UiNode,
    parent_path: &str,
    rect: UiRect,
    selected: &str,
    tabs: &[crate::model::UiTab],
) -> Vec<UiLayoutNode> {
    let header_height = tab_view_header_height(node);
    let content = UiRect::new(
        rect.x + node.style.padding,
        rect.y + header_height + node.style.padding,
        (rect.width - node.style.padding * 2.0).max(0.0),
        (rect.height - header_height - node.style.padding * 2.0).max(0.0),
    );
    let selected = selected_tab_id(selected, tabs, &node.children);
    node.children
        .iter()
        .enumerate()
        .filter(|(_, child)| child.id.as_deref() == Some(selected.as_str()))
        .map(|(index, child)| {
            let path = child_path(parent_path, child, index);
            layout_node(child, path, content, index)
        })
        .collect()
}

fn layout_row(node: &UiNode, parent_path: &str, rect: UiRect) -> Vec<UiLayoutNode> {
    let content = rect.inset(node.style.padding);
    let mut cursor_x = content.x;
    let mut children = Vec::with_capacity(node.children.len());

    for (index, child) in node.children.iter().enumerate() {
        let child_rect = UiRect::new(cursor_x, content.y, content.width, content.height);
        let path = child_path(parent_path, child, index);
        let layout = layout_node(child, path, child_rect, index);
        cursor_x = layout.rect.x + layout.rect.width + node.style.gap;
        children.push(layout);
    }

    children
}

fn layout_stack(node: &UiNode, parent_path: &str, rect: UiRect) -> Vec<UiLayoutNode> {
    let content = rect.inset(node.style.padding);
    node.children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let path = child_path(parent_path, child, index);
            layout_node(child, path, content, index)
        })
        .collect()
}

