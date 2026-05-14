use crate::model::LayoutNode;

pub fn hit_test<T>(node: &LayoutNode<T>, x: f32, y: f32) -> Option<String> {
    if !node.rect.contains(x, y) {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}

pub fn find_layout_node<'a, T>(node: &'a LayoutNode<T>, path: &str) -> Option<&'a LayoutNode<T>> {
    if node.path == path {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_layout_node(child, path) {
            return Some(found);
        }
    }
    None
}

pub fn flatten_layout<'a, T>(node: &'a LayoutNode<T>) -> Vec<&'a LayoutNode<T>> {
    let mut out = Vec::new();
    flatten_inner(node, &mut out);
    out
}

fn flatten_inner<'a, T>(node: &'a LayoutNode<T>, out: &mut Vec<&'a LayoutNode<T>>) {
    out.push(node);
    for child in &node.children {
        flatten_inner(child, out);
    }
}
