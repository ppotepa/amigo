use amigo_ui_layout::{
    LayoutElement, LayoutKind, LayoutLeafKind, LayoutNode, LayoutStyle, LayoutTab, LayoutViewport,
    LayoutViewportScaling,
};

pub fn build_ui_layout_tree(viewport: UiViewportSize, document: &UiOverlayDocument) -> UiLayoutNode {
    let document_viewport = document.viewport.map(|vp| {
        (
            LayoutViewport::new(vp.width, vp.height),
            match vp.scaling {
                UiOverlayViewportScaling::Expand => LayoutViewportScaling::Expand,
                UiOverlayViewportScaling::Fixed => LayoutViewportScaling::Fixed,
                UiOverlayViewportScaling::Fit => LayoutViewportScaling::Fit,
            },
        )
    });
    let root = overlay_node_to_layout(&document.root);
    let layout = amigo_ui_layout::compute_layout(
        &document.entity_name,
        LayoutViewport::new(viewport.width, viewport.height),
        &root,
        document_viewport,
    );
    let mut ui_layout = layout_to_ui_layout_node(layout);
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    apply_overlay_paths(
        &mut ui_layout,
        &format!("{}.{}", document.entity_name, root_segment),
        0,
    );
    if let Some(vp) = document.viewport {
        if vp.scaling == UiOverlayViewportScaling::Fit {
            let scale = (viewport.width / vp.width.max(1.0)).min(viewport.height / vp.height.max(1.0));
            scale_styles(&mut ui_layout, scale.max(0.0));
        }
    }
    ui_layout
}

fn overlay_node_to_layout(node: &UiOverlayNode) -> LayoutElement<UiOverlayNode> {
    LayoutElement {
        id: node.id.clone(),
        kind: overlay_kind_to_layout_kind(&node.kind),
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
        children: node.children.iter().map(overlay_node_to_layout).collect(),
    }
}

fn overlay_kind_to_layout_kind(kind: &UiOverlayNodeKind) -> LayoutKind {
    match kind {
        UiOverlayNodeKind::Panel => LayoutKind::Panel,
        UiOverlayNodeKind::GroupBox { label, .. } => LayoutKind::GroupBox {
            label: label.clone(),
        },
        UiOverlayNodeKind::Row => LayoutKind::Row,
        UiOverlayNodeKind::Column => LayoutKind::Column,
        UiOverlayNodeKind::Stack => LayoutKind::Stack,
        UiOverlayNodeKind::Text { content, .. } => LayoutKind::Leaf(LayoutLeafKind::Text {
            content: content.clone(),
        }),
        UiOverlayNodeKind::Button { text, .. } => {
            LayoutKind::Leaf(LayoutLeafKind::Button { text: text.clone() })
        }
        UiOverlayNodeKind::ProgressBar { .. } => LayoutKind::Leaf(LayoutLeafKind::ProgressBar),
        UiOverlayNodeKind::Slider { .. } => LayoutKind::Leaf(LayoutLeafKind::Slider),
        UiOverlayNodeKind::Toggle { text, .. } => LayoutKind::Leaf(LayoutLeafKind::Toggle {
            text: text.clone(),
        }),
        UiOverlayNodeKind::OptionSet { options, .. } => LayoutKind::Leaf(LayoutLeafKind::OptionSet {
            option_count: options.len(),
        }),
        UiOverlayNodeKind::Dropdown {
            options, expanded, ..
        } => LayoutKind::Leaf(LayoutLeafKind::Dropdown {
            option_count: options.len(),
            expanded: *expanded,
        }),
        UiOverlayNodeKind::TabView { selected, tabs, .. } => LayoutKind::TabView {
            selected: selected.clone(),
            tabs: tabs
                .iter()
                .map(|tab| LayoutTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
        },
        UiOverlayNodeKind::ColorPickerRgb { .. } => LayoutKind::Leaf(LayoutLeafKind::ColorPickerRgb),
        UiOverlayNodeKind::CurveEditor { .. } => LayoutKind::Leaf(LayoutLeafKind::CurveEditor),
        UiOverlayNodeKind::Spacer => LayoutKind::Leaf(LayoutLeafKind::Spacer),
    }
}

fn layout_to_ui_layout_node(node: LayoutNode<UiOverlayNode>) -> UiLayoutNode {
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

fn apply_overlay_paths(node: &mut UiLayoutNode, parent_path: &str, depth: usize) {
    node.path = parent_path.to_owned();
    for (index, child) in node.children.iter_mut().enumerate() {
        let segment = child
            .node
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{depth}-{index}", kind_slug(&child.node.kind)));
        let path = format!("{parent_path}.{segment}");
        apply_overlay_paths(child, &path, depth + 1);
    }
}

fn scale_styles(node: &mut UiLayoutNode, scale: f32) {
    node.node.style.padding *= scale;
    node.node.style.gap *= scale;
    node.node.style.border_width *= scale;
    node.node.style.border_radius *= scale;
    node.node.style.font_size *= scale;
    for child in &mut node.children {
        scale_styles(child, scale);
    }
}
