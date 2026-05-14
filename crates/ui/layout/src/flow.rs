use crate::measure::{
    group_box_label_height, measure_element, selected_tab_id, tab_view_header_height,
};
use crate::model::{LayoutElement, LayoutKind, LayoutLeafKind, LayoutNode, LayoutRect};

pub(crate) fn layout_node<T: Clone>(
    parent_path: &str,
    node: &LayoutElement<T>,
    rect: LayoutRect,
    segment: String,
    depth_index: usize,
) -> LayoutNode<T> {
    let path = format!("{parent_path}.{segment}");
    let content = rect.inset(node.style.padding.max(0.0));
    let gap = node.style.gap.max(0.0);

    let children_rects = match &node.kind {
        LayoutKind::Row => layout_row_children(node, content, gap),
        LayoutKind::Stack => layout_stack_children(node, content),
        LayoutKind::GroupBox { .. } => layout_group_box_children(node, content, gap),
        LayoutKind::TabView { selected, tabs } => {
            layout_tab_view_children(node, rect, selected, tabs)
        }
        LayoutKind::Column | LayoutKind::Panel => layout_column_children(node, content, gap),
        LayoutKind::Leaf(_) => Vec::new(),
    };

    let mut layout_children = Vec::with_capacity(children_rects.len());
    for (index, (child, child_rect)) in children_rects.into_iter().enumerate() {
        let child_segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{depth_index}-{index}", kind_slug(&child.kind)));
        layout_children.push(layout_node(
            &path,
            child,
            child_rect,
            child_segment,
            depth_index + 1,
        ));
    }

    LayoutNode {
        path,
        rect,
        data: node.data.clone(),
        children: layout_children,
    }
}

fn kind_slug(kind: &LayoutKind) -> &'static str {
    match kind {
        LayoutKind::Panel => "panel",
        LayoutKind::GroupBox { .. } => "group-box",
        LayoutKind::Row => "row",
        LayoutKind::Column => "column",
        LayoutKind::Stack => "stack",
        LayoutKind::TabView { .. } => "tab-view",
        LayoutKind::Leaf(leaf) => match leaf {
            LayoutLeafKind::Text { .. } => "text",
            LayoutLeafKind::Button { .. } => "button",
            LayoutLeafKind::ProgressBar => "progress-bar",
            LayoutLeafKind::Slider => "slider",
            LayoutLeafKind::Toggle { .. } => "toggle",
            LayoutLeafKind::OptionSet { .. } => "option-set",
            LayoutLeafKind::Dropdown { .. } => "dropdown",
            LayoutLeafKind::ColorPickerRgb => "color-picker-rgb",
            LayoutLeafKind::CurveEditor => "curve-editor",
            LayoutLeafKind::Spacer => "spacer",
        },
    }
}

fn layout_group_box_children<'a, T>(
    node: &'a LayoutElement<T>,
    content: LayoutRect,
    gap: f32,
) -> Vec<(&'a LayoutElement<T>, LayoutRect)> {
    let label_height = group_box_label_height(node.style.font_size);
    let content = LayoutRect::new(
        content.x,
        content.y + label_height,
        content.width,
        (content.height - label_height).max(0.0),
    );
    layout_column_children(node, content, gap)
}

fn layout_tab_view_children<'a, T>(
    node: &'a LayoutElement<T>,
    rect: LayoutRect,
    selected: &str,
    tabs: &[crate::model::LayoutTab],
) -> Vec<(&'a LayoutElement<T>, LayoutRect)> {
    let selected = selected_tab_id(selected, tabs, &node.children);
    let header_height = tab_view_header_height(node.style.font_size, node.style.padding.max(0.0));
    let padding = node.style.padding.max(0.0);
    let content = LayoutRect::new(
        rect.x + padding,
        rect.y + header_height + padding,
        (rect.width - padding * 2.0).max(0.0),
        (rect.height - header_height - padding * 2.0).max(0.0),
    );

    node.children
        .iter()
        .filter(|child| child.id.as_deref() == Some(selected.as_str()))
        .map(|child| (child, content))
        .collect()
}

fn layout_column_children<'a, T>(
    node: &'a LayoutElement<T>,
    content: LayoutRect,
    gap: f32,
) -> Vec<(&'a LayoutElement<T>, LayoutRect)> {
    let mut cursor = content.y;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_element(child);
        let width = child
            .style
            .width
            .unwrap_or(content.width.max(measured.0))
            .max(0.0);
        let height = child.style.height.unwrap_or(measured.1).max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = cursor + child.style.top.unwrap_or(0.0);
        laid_out.push((child, LayoutRect::new(x, y, width, height)));
        cursor = y + height + gap;
    }
    laid_out
}

fn layout_row_children<'a, T>(
    node: &'a LayoutElement<T>,
    content: LayoutRect,
    gap: f32,
) -> Vec<(&'a LayoutElement<T>, LayoutRect)> {
    let mut cursor = content.x;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_element(child);
        let width = child.style.width.unwrap_or(measured.0).max(0.0);
        let height = child
            .style
            .height
            .unwrap_or(content.height.max(measured.1))
            .max(0.0);
        let x = cursor + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((child, LayoutRect::new(x, y, width, height)));
        cursor = x + width + gap;
    }
    laid_out
}

fn layout_stack_children<'a, T>(
    node: &'a LayoutElement<T>,
    content: LayoutRect,
) -> Vec<(&'a LayoutElement<T>, LayoutRect)> {
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_element(child);
        let width = child
            .style
            .width
            .unwrap_or(content.width.max(measured.0))
            .max(0.0);
        let height = child
            .style
            .height
            .unwrap_or(content.height.max(measured.1))
            .max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((
            child,
            LayoutRect::new(x, y, width.min(content.width), height.min(content.height)),
        ));
    }
    laid_out
}
