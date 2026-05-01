fn layout_node(
    entity_name: &str,
    node: &UiOverlayNode,
    rect: UiRect,
    segment: String,
    depth_index: usize,
) -> UiLayoutNode {
    let path = format!("{entity_name}.{segment}");
    let content = rect.inset(node.style.padding.max(0.0));
    let gap = node.style.gap.max(0.0);
    let children = match &node.kind {
        UiOverlayNodeKind::Row => layout_row_children(node, content, gap),
        UiOverlayNodeKind::Stack => layout_stack_children(node, content),
        UiOverlayNodeKind::GroupBox { .. } => layout_group_box_children(node, content, gap),
        UiOverlayNodeKind::TabView { selected, tabs, .. } => {
            layout_tab_view_children(node, rect, selected, tabs)
        }
        UiOverlayNodeKind::Column | UiOverlayNodeKind::Panel => {
            layout_column_children(node, content, gap)
        }
        UiOverlayNodeKind::Text { .. }
        | UiOverlayNodeKind::Button { .. }
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. }
        | UiOverlayNodeKind::CurveEditor { .. }
        | UiOverlayNodeKind::Spacer => Vec::new(),
    };

    let node_with_children = UiOverlayNode {
        id: node.id.clone(),
        kind: node.kind.clone(),
        style: node.style.clone(),
        children: node.children.clone(),
    };

    let mut layout_children = Vec::with_capacity(children.len());
    for (index, (child, child_rect)) in children.into_iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{depth_index}-{index}", kind_slug(&child.kind)));
        layout_children.push(layout_node(
            &path,
            child,
            child_rect,
            segment,
            depth_index + 1,
        ));
    }

    UiLayoutNode {
        path,
        rect,
        node: node_with_children,
        children: layout_children,
    }
}

fn layout_group_box_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
    gap: f32,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let label_height = group_box_label_height(node);
    let content = UiRect::new(
        content.x,
        content.y + label_height,
        content.width,
        (content.height - label_height).max(0.0),
    );
    layout_column_children(node, content, gap)
}

fn layout_tab_view_children<'a>(
    node: &'a UiOverlayNode,
    rect: UiRect,
    selected: &str,
    tabs: &[UiOverlayTab],
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let selected = selected_tab_id(selected, tabs, &node.children);
    let header_height = tab_view_header_height(node);
    let content = UiRect::new(
        rect.x + node.style.padding.max(0.0),
        rect.y + header_height + node.style.padding.max(0.0),
        (rect.width - node.style.padding.max(0.0) * 2.0).max(0.0),
        (rect.height - header_height - node.style.padding.max(0.0) * 2.0).max(0.0),
    );
    node.children
        .iter()
        .filter(|child| child.id.as_deref() == Some(selected.as_str()))
        .map(|child| (child, content))
        .collect()
}

fn layout_column_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
    gap: f32,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut cursor = content.y;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child
            .style
            .width
            .unwrap_or_else(|| default_child_width_for_column(child, content.width, measured.x))
            .max(0.0);
        let height = resolved_child_height_for_column(child, measured.y).max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = cursor + child.style.top.unwrap_or(0.0);
        laid_out.push((child, UiRect::new(x, y, width, height)));
        cursor = y + height + gap;
    }
    laid_out
}

fn layout_row_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
    gap: f32,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut cursor = content.x;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child.style.width.unwrap_or(measured.x).max(0.0);
        let height = resolved_child_height_for_row(child, content.height, measured.y).max(0.0);
        let x = cursor + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((child, UiRect::new(x, y, width, height)));
        cursor = x + width + gap;
    }
    laid_out
}

fn layout_stack_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child
            .style
            .width
            .unwrap_or(content.width.max(measured.x))
            .max(0.0);
        let height = child
            .style
            .height
            .unwrap_or(content.height.max(measured.y))
            .max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((
            child,
            UiRect::new(x, y, width.min(content.width), height.min(content.height)),
        ));
    }
    laid_out
}

fn resolved_child_height_for_column(child: &UiOverlayNode, measured_height: f32) -> f32 {
    child.style.height.unwrap_or(measured_height)
}

fn resolved_child_height_for_row(
    child: &UiOverlayNode,
    content_height: f32,
    measured_height: f32,
) -> f32 {
    let default_height = default_child_height_for_row(child, content_height, measured_height);
    child.style.height.unwrap_or(default_height)
}

