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

