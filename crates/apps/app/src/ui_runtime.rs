use super::*;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedUiOverlayDocument {
    pub(crate) overlay: UiOverlayDocument,
    pub(crate) click_bindings: BTreeMap<String, UiEventBinding>,
}

pub(crate) fn resolve_ui_overlay_documents(
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
) -> Vec<ResolvedUiOverlayDocument> {
    let snapshot = ui_state_service.snapshot();
    let mut documents = ui_scene_service
        .commands()
        .into_iter()
        .filter_map(|command| {
            resolve_ui_overlay_document(&command.entity_name, &command.document, &snapshot)
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| document.overlay.layer);
    documents
}

fn resolve_ui_overlay_document(
    entity_name: &str,
    document: &RuntimeUiDocument,
    snapshot: &UiStateSnapshot,
) -> Option<ResolvedUiOverlayDocument> {
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    let root_path = format!("{entity_name}.{root_segment}");
    let mut click_bindings = BTreeMap::new();
    let root = resolve_ui_overlay_node(&document.root, &root_path, snapshot, &mut click_bindings)?;

    Some(ResolvedUiOverlayDocument {
        overlay: UiOverlayDocument {
            entity_name: entity_name.to_owned(),
            layer: resolve_ui_overlay_layer(&document.target),
            root,
        },
        click_bindings,
    })
}

fn resolve_ui_overlay_layer(target: &RuntimeUiTarget) -> UiOverlayLayer {
    match target.layer() {
        RuntimeUiLayer::Background => UiOverlayLayer::Background,
        RuntimeUiLayer::Hud => UiOverlayLayer::Hud,
        RuntimeUiLayer::Menu => UiOverlayLayer::Menu,
        RuntimeUiLayer::Debug => UiOverlayLayer::Debug,
    }
}

fn resolve_ui_overlay_node(
    node: &RuntimeUiNode,
    path: &str,
    snapshot: &UiStateSnapshot,
    click_bindings: &mut BTreeMap<String, UiEventBinding>,
) -> Option<UiOverlayNode> {
    if snapshot
        .visibility_overrides
        .get(path)
        .copied()
        .unwrap_or(true)
        == false
    {
        return None;
    }

    let kind = match &node.kind {
        RuntimeUiNodeKind::Panel => UiOverlayNodeKind::Panel,
        RuntimeUiNodeKind::Row => UiOverlayNodeKind::Row,
        RuntimeUiNodeKind::Column => UiOverlayNodeKind::Column,
        RuntimeUiNodeKind::Stack => UiOverlayNodeKind::Stack,
        RuntimeUiNodeKind::Text { content, font } => UiOverlayNodeKind::Text {
            content: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| content.clone()),
            font: font.clone(),
        },
        RuntimeUiNodeKind::Button { text, font } => UiOverlayNodeKind::Button {
            text: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| text.clone()),
            font: font.clone(),
        },
        RuntimeUiNodeKind::ProgressBar { value } => UiOverlayNodeKind::ProgressBar {
            value: snapshot
                .value_overrides
                .get(path)
                .copied()
                .unwrap_or(*value)
                .clamp(0.0, 1.0),
        },
        RuntimeUiNodeKind::Spacer => UiOverlayNodeKind::Spacer,
    };

    let mut children = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", runtime_ui_node_kind_slug(&child.kind)));
        let child_path = format!("{path}.{segment}");
        if let Some(child) = resolve_ui_overlay_node(child, &child_path, snapshot, click_bindings) {
            children.push(child);
        }
    }

    if let Some(binding) = node.events.on_click.as_ref() {
        click_bindings.insert(path.to_owned(), binding.clone());
    }

    Some(UiOverlayNode {
        id: node.id.clone(),
        kind,
        style: resolve_ui_overlay_style(&node.style),
        children,
    })
}

fn runtime_ui_node_kind_slug(kind: &RuntimeUiNodeKind) -> &'static str {
    match kind {
        RuntimeUiNodeKind::Panel => "panel",
        RuntimeUiNodeKind::Row => "row",
        RuntimeUiNodeKind::Column => "column",
        RuntimeUiNodeKind::Stack => "stack",
        RuntimeUiNodeKind::Text { .. } => "text",
        RuntimeUiNodeKind::Button { .. } => "button",
        RuntimeUiNodeKind::ProgressBar { .. } => "progress-bar",
        RuntimeUiNodeKind::Spacer => "spacer",
    }
}

fn resolve_ui_overlay_style(style: &RuntimeUiStyle) -> UiOverlayStyle {
    UiOverlayStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: style.background,
        color: style.color,
        border_color: style.border_color,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
    }
}

pub(crate) fn hit_test_ui_layout(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if x < node.rect.x
        || y < node.rect.y
        || x > node.rect.x + node.rect.width
        || y > node.rect.y + node.rect.height
    {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_ui_layout(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}
