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
        UiNodeKind::Stack | UiNodeKind::Panel => layout_stack(node, &path, rect),
        UiNodeKind::Text { .. }
        | UiNodeKind::Button { .. }
        | UiNodeKind::ProgressBar { .. }
        | UiNodeKind::Spacer => Vec::new(),
    };

    UiLayoutNode {
        path,
        rect,
        node: node.clone(),
        children,
    }
}

fn layout_column(node: &UiNode, parent_path: &str, rect: UiRect) -> Vec<UiLayoutNode> {
    let content = rect.inset(node.style.padding);
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
        UiNodeKind::Text { .. } => node.style.font_size * 1.4,
        UiNodeKind::Button { .. } => 40.0,
        UiNodeKind::ProgressBar { .. } => 18.0,
        UiNodeKind::Spacer => 0.0,
        UiNodeKind::Panel | UiNodeKind::Stack => available.height.max(0.0),
        UiNodeKind::Column => measure_column_height(node, available),
        UiNodeKind::Row => measure_row_height(node, available),
    };

    UiRect::new(available.x, available.y, fallback_width, fallback_height)
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

#[cfg(test)]
mod tests {
    use super::{UiLayoutService, compute_layout, hit_test};
    use crate::model::{
        UiDocument, UiEventBinding, UiEvents, UiLayer, UiNode, UiNodeKind, UiRect, UiStyle,
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
}
