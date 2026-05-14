use super::*;

mod editor;
mod showcase;

fn find_layout_node_by_path_suffix<'a>(
    node: &'a OverlayUiLayoutNode,
    suffix: &str,
) -> Option<&'a OverlayUiLayoutNode> {
    if node.path.ends_with(suffix) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_layout_node_by_path_suffix(child, suffix) {
            return Some(found);
        }
    }
    None
}
