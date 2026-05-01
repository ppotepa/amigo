use crate::model::{UiDocument, UiLayoutNode, UiNode, UiNodeKind, UiRect};

#[derive(Debug, Default, Clone, Copy)]
pub struct UiLayoutService;

impl UiLayoutService {
    pub fn compute(&self, document: &UiDocument, viewport: UiRect) -> UiLayoutNode {
        compute_layout(document, viewport)
    }

    pub fn hit_test(&self, layout: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
        hit_test(layout, x, y)
    }
}

pub fn compute_layout(document: &UiDocument, viewport: UiRect) -> UiLayoutNode {
    let root_path = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    layout_node(&document.root, root_path, viewport, 0)
}

pub fn hit_test(layout: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
    hit_test_node(layout, x, y)
}

fn hit_test_node(node: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
    if !node.rect.contains(x, y) {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_node(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}

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

fn resolve_rect(node: &UiNode, available: UiRect, index: usize) -> UiRect {
    let default_size = default_node_size(node, available, index);
    let width = node.style.width.unwrap_or(default_size.width).max(0.0);
    let height = node.style.height.unwrap_or(default_size.height).max(0.0);
    let x = available.x + node.style.left.unwrap_or(0.0);
    let y = available.y + node.style.top.unwrap_or(0.0);

    UiRect::new(x, y, width, height)
}

fn default_node_size(node: &UiNode, available: UiRect, _index: usize) -> UiRect {
    let fallback_width = available.width.max(0.0);
    let fallback_height = match &node.kind {
        UiNodeKind::Text { content, .. } => measure_text_height(
            content,
            node.style.width.unwrap_or(fallback_width),
            node.style.font_size,
            node.style.word_wrap,
            node.style.fit_to_width,
        ),
        UiNodeKind::Button { .. } => 40.0,
        UiNodeKind::GroupBox { .. } => measure_group_box_height(node, available),
        UiNodeKind::ProgressBar { .. } => 18.0,
        UiNodeKind::Slider { .. } => 24.0,
        UiNodeKind::Toggle { .. } => 36.0,
        UiNodeKind::OptionSet { .. } => 38.0,
        UiNodeKind::Dropdown { .. } => 38.0,
        UiNodeKind::TabView { selected, tabs, .. } => {
            measure_tab_view_height(node, available, selected, tabs)
        }
        UiNodeKind::ColorPickerRgb { .. } => 118.0,
        UiNodeKind::Spacer => 0.0,
        UiNodeKind::Panel | UiNodeKind::Stack => available.height.max(0.0),
        UiNodeKind::Column => measure_column_height(node, available),
        UiNodeKind::Row => measure_row_height(node, available),
    };

    UiRect::new(available.x, available.y, fallback_width, fallback_height)
}

fn measure_group_box_height(node: &UiNode, available: UiRect) -> f32 {
    group_box_label_height(node) + measure_column_height(node, available)
}

fn measure_tab_view_height(
    node: &UiNode,
    available: UiRect,
    selected: &str,
    tabs: &[crate::model::UiTab],
) -> f32 {
    let selected = selected_tab_id(selected, tabs, &node.children);
    let panel_height = node
        .children
        .iter()
        .find(|child| child.id.as_deref() == Some(selected.as_str()))
        .map(|child| {
            let default = default_node_size(child, available, 0);
            child.style.height.unwrap_or(default.height).max(0.0)
        })
        .unwrap_or(0.0);
    tab_view_header_height(node) + node.style.padding * 2.0 + panel_height
}

fn group_box_label_height(node: &UiNode) -> f32 {
    node.style.font_size.max(8.0) * 1.2
}

fn tab_view_header_height(node: &UiNode) -> f32 {
    (node.style.font_size.max(14.0) * 1.2 + node.style.padding * 2.0).max(38.0)
}

fn selected_tab_id(selected: &str, tabs: &[crate::model::UiTab], children: &[UiNode]) -> String {
    if tabs.iter().any(|tab| tab.id == selected) {
        return selected.to_owned();
    }
    tabs.first()
        .map(|tab| tab.id.clone())
        .or_else(|| children.iter().find_map(|child| child.id.clone()))
        .unwrap_or_default()
}

fn measure_column_height(node: &UiNode, available: UiRect) -> f32 {
    let mut total = node.style.padding * 2.0;
    let mut iter = node.children.iter().peekable();

    while let Some(child) = iter.next() {
        let default = default_node_size(child, available, 0);
        let extent = child.style.height.unwrap_or(default.height);
        total += extent.max(0.0);

        if iter.peek().is_some() {
            total += node.style.gap.max(0.0);
        }
    }

    total
}

fn measure_row_height(node: &UiNode, available: UiRect) -> f32 {
    let max_child_height = node
        .children
        .iter()
        .map(|child| {
            let default = default_node_size(child, available, 0);
            child.style.height.unwrap_or(default.height).max(0.0)
        })
        .fold(0.0, f32::max);

    node.style.padding * 2.0 + max_child_height
}

fn child_path(parent_path: &str, child: &UiNode, index: usize) -> String {
    let segment = child
        .id
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}-{index}", child.kind.label()));
    format!("{parent_path}.{segment}")
}

fn measure_text_height(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> f32 {
    let effective_font_size =
        measure_effective_font_size(content, max_width, font_size, word_wrap, fit_to_width);
    let line_height = effective_font_size.max(8.0) * 1.2;
    let line_count = measure_wrapped_line_count(content, effective_font_size, max_width, word_wrap);
    (line_count.max(1) as f32) * line_height
}

fn measure_effective_font_size(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> f32 {
    let mut effective_font_size = font_size.max(8.0);
    if fit_to_width && !word_wrap && max_width > 0.0 {
        let width = measure_text_line_width(content, effective_font_size);
        if width > max_width {
            effective_font_size = (effective_font_size * (max_width / width))
                .max(8.0)
                .min(effective_font_size);
        }
    }
    effective_font_size
}

fn measure_wrapped_line_count(
    content: &str,
    font_size: f32,
    max_width: f32,
    word_wrap: bool,
) -> usize {
    if !(word_wrap && max_width > 0.0) {
        return content.split('\n').count().max(1);
    }

    let mut lines = 0usize;
    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines += 1;
            continue;
        }

        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };

            if measure_text_line_width(&candidate, font_size) <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                lines += 1;
                current.clear();
            }

            if measure_text_line_width(word, font_size) <= max_width {
                current = word.to_owned();
                continue;
            }

            let mut fragment = String::new();
            for ch in word.chars() {
                let candidate = format!("{fragment}{ch}");
                if !fragment.is_empty()
                    && measure_text_line_width(&candidate, font_size) > max_width
                {
                    lines += 1;
                    fragment.clear();
                }
                fragment.push(ch);
            }
            current = fragment;
        }

        if !current.is_empty() {
            lines += 1;
        }
    }

    lines.max(1)
}

fn measure_text_line_width(content: &str, font_size: f32) -> f32 {
    let effective_font_size = font_size.max(8.0);
    let advance = effective_font_size * (6.0 / 7.0);
    content.chars().count() as f32 * advance
}

#[cfg(test)]
mod tests {
    use super::{UiLayoutService, compute_layout, hit_test};
    use crate::model::{
        UiDocument, UiEventBinding, UiEvents, UiLayer, UiNode, UiNodeKind, UiRect, UiStyle, UiTab,
    };

    #[test]
    fn computes_column_layout() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column)
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 8.0,
                    ..UiStyle::default()
                })
                .with_children(vec![
                    UiNode::new(UiNodeKind::Text {
                        content: "Title".to_owned(),
                        font: None,
                    })
                    .with_id("title")
                    .with_style(UiStyle {
                        height: Some(20.0),
                        ..UiStyle::default()
                    }),
                    UiNode::new(UiNodeKind::Button {
                        text: "Click".to_owned(),
                        font: None,
                    })
                    .with_id("button")
                    .with_style(UiStyle {
                        height: Some(40.0),
                        ..UiStyle::default()
                    }),
                ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(layout.rect, UiRect::new(24.0, 24.0, 200.0, 100.0));
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.children[0].path, "root.title");
        assert_eq!(
            layout.children[0].rect,
            UiRect::new(40.0, 40.0, 168.0, 20.0)
        );
        assert_eq!(
            layout.children[1].rect,
            UiRect::new(40.0, 68.0, 168.0, 40.0)
        );
    }

    #[test]
    fn hit_tests_button() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column).with_children(vec![
                UiNode::new(UiNodeKind::Button {
                    text: "Emit".to_owned(),
                    font: None,
                })
                .with_id("action-button")
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(220.0),
                    height: Some(42.0),
                    ..UiStyle::default()
                })
                .with_events(UiEvents {
                    on_click: Some(UiEventBinding::new(
                        "playground-2d.ui-preview.button-clicked",
                        Vec::new(),
                    )),
                    on_change: None,
                }),
            ]),
        );

        let service = UiLayoutService;
        let layout = service.compute(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(
            hit_test(&layout, 40.0, 40.0).as_deref(),
            Some("root.action-button")
        );
        assert_eq!(hit_test(&layout, 400.0, 400.0), None);
    }

    #[test]
    fn wrapped_text_increases_column_layout_height() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::Column)
                .with_style(UiStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 8.0,
                    ..UiStyle::default()
                })
                .with_children(vec![
                    UiNode::new(UiNodeKind::Text {
                        content: "grounded=false vx=120 vy=-10 anim=run".to_owned(),
                        font: None,
                    })
                    .with_id("debug")
                    .with_style(UiStyle {
                        width: Some(168.0),
                        font_size: 14.0,
                        word_wrap: true,
                        ..UiStyle::default()
                    }),
                    UiNode::new(UiNodeKind::Text {
                        content: "READY".to_owned(),
                        font: None,
                    })
                    .with_id("message"),
                ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert!(layout.children[0].rect.height > 14.0 * 1.4);
        assert!(
            layout.children[1].rect.y >= layout.children[0].rect.y + layout.children[0].rect.height
        );
    }

    #[test]
    fn tab_view_lays_out_only_selected_panel() {
        let document = UiDocument::screen_space(
            UiLayer::Hud,
            UiNode::new(UiNodeKind::TabView {
                selected: "settings".to_owned(),
                tabs: vec![
                    UiTab {
                        id: "overview".to_owned(),
                        label: "Overview".to_owned(),
                    },
                    UiTab {
                        id: "settings".to_owned(),
                        label: "Settings".to_owned(),
                    },
                ],
                font: None,
            })
            .with_id("tabs")
            .with_style(UiStyle {
                width: Some(300.0),
                height: Some(180.0),
                padding: 4.0,
                ..UiStyle::default()
            })
            .with_children(vec![
                UiNode::new(UiNodeKind::Text {
                    content: "Overview panel".to_owned(),
                    font: None,
                })
                .with_id("overview"),
                UiNode::new(UiNodeKind::Text {
                    content: "Settings panel".to_owned(),
                    font: None,
                })
                .with_id("settings"),
            ]),
        );

        let layout = compute_layout(&document, UiRect::new(0.0, 0.0, 1280.0, 720.0));

        assert_eq!(layout.children.len(), 1);
        assert_eq!(layout.children[0].path, "tabs.settings");
        assert!(layout.children[0].rect.y >= 38.0);
    }
}
