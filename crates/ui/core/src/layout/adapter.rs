use amigo_ui_layout::{
    LayoutElement, LayoutKind, LayoutLeafKind, LayoutNode, LayoutStyle, LayoutTab, LayoutViewport,
};

fn ui_document_to_layout(document: &UiDocument) -> LayoutElement<UiNode> {
    ui_node_to_layout(&document.root)
}

fn ui_node_to_layout(node: &UiNode) -> LayoutElement<UiNode> {
    LayoutElement {
        id: node.id.clone(),
        kind: ui_kind_to_layout_kind(&node.kind),
        style: LayoutStyle {
            left: node.style.left,
            top: node.style.top,
            right: node.style.right,
            bottom: node.style.bottom,
            width: node.style.width,
            height: node.style.height,
            padding: node.style.padding,
            gap: node.style.gap,
            border_width: node.style.border_width,
            border_radius: node.style.border_radius,
            font_size: node.style.font_size,
            word_wrap: node.style.word_wrap,
            fit_to_width: node.style.fit_to_width,
        },
        data: node.clone(),
        children: node.children.iter().map(ui_node_to_layout).collect(),
    }
}

fn ui_kind_to_layout_kind(kind: &UiNodeKind) -> LayoutKind {
    match kind {
        UiNodeKind::Panel => LayoutKind::Panel,
        UiNodeKind::GroupBox { label, .. } => LayoutKind::GroupBox {
            label: label.clone(),
        },
        UiNodeKind::Row => LayoutKind::Row,
        UiNodeKind::Column => LayoutKind::Column,
        UiNodeKind::Stack => LayoutKind::Stack,
        UiNodeKind::Text { content, .. } => LayoutKind::Leaf(LayoutLeafKind::Text {
            content: content.clone(),
        }),
        UiNodeKind::Button { text, .. } => {
            LayoutKind::Leaf(LayoutLeafKind::Button { text: text.clone() })
        }
        UiNodeKind::ProgressBar { .. } => LayoutKind::Leaf(LayoutLeafKind::ProgressBar),
        UiNodeKind::Slider { .. } => LayoutKind::Leaf(LayoutLeafKind::Slider),
        UiNodeKind::Toggle { text, .. } => LayoutKind::Leaf(LayoutLeafKind::Toggle {
            text: text.clone(),
        }),
        UiNodeKind::OptionSet { options, .. } => LayoutKind::Leaf(LayoutLeafKind::OptionSet {
            option_count: options.len(),
        }),
        UiNodeKind::Dropdown { options, .. } => LayoutKind::Leaf(LayoutLeafKind::Dropdown {
            option_count: options.len(),
            expanded: false,
        }),
        UiNodeKind::TabView { selected, tabs, .. } => LayoutKind::TabView {
            selected: selected.clone(),
            tabs: tabs
                .iter()
                .map(|tab| LayoutTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
        },
        UiNodeKind::ColorPickerRgb { .. } => LayoutKind::Leaf(LayoutLeafKind::ColorPickerRgb),
        UiNodeKind::CurveEditor { .. } => LayoutKind::Leaf(LayoutLeafKind::CurveEditor),
        UiNodeKind::Spacer => LayoutKind::Leaf(LayoutLeafKind::Spacer),
    }
}

fn layout_to_ui_layout_node(node: LayoutNode<UiNode>) -> UiLayoutNode {
    UiLayoutNode {
        path: node.path,
        rect: UiRect::new(node.rect.x, node.rect.y, node.rect.width, node.rect.height),
        node: node.data,
        children: node
            .children
            .into_iter()
            .map(layout_to_ui_layout_node)
            .collect(),
    }
}

fn apply_stable_paths(node: &mut UiLayoutNode) {
    for (index, child) in node.children.iter_mut().enumerate() {
        let segment = child
            .node
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", child.node.kind.label()));
        child.path = format!("{}.{}", node.path, segment);
        apply_stable_paths(child);
    }
}

fn compute_layout_with_kernel(document: &UiDocument, viewport: UiRect) -> UiLayoutNode {
    let root = ui_document_to_layout(document);
    let root_path = document.root.id.clone().unwrap_or_else(|| "root".to_owned());
    let layout = amigo_ui_layout::compute_layout(
        "",
        LayoutViewport::new(viewport.width, viewport.height),
        &root,
        None,
    );
    let mut ui_layout = layout_to_ui_layout_node(layout);
    ui_layout.path = root_path;
    apply_stable_paths(&mut ui_layout);
    ui_layout
}
