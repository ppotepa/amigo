use super::*;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedUiOverlayDocument {
    pub(crate) overlay: UiOverlayDocument,
    pub(crate) click_bindings: BTreeMap<String, UiEventBinding>,
    pub(crate) change_bindings: BTreeMap<String, UiEventBinding>,
}

pub(crate) fn resolve_ui_overlay_documents(
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    ui_theme_service: &UiThemeService,
) -> Vec<ResolvedUiOverlayDocument> {
    let snapshot = ui_state_service.snapshot();
    let active_theme = ui_theme_service.active_theme();
    let mut documents = ui_scene_service
        .commands()
        .into_iter()
        .filter_map(|command| {
            resolve_ui_overlay_document(
                &command.entity_name,
                &command.document,
                &snapshot,
                active_theme.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| document.overlay.layer);
    documents
}

fn resolve_ui_overlay_document(
    entity_name: &str,
    document: &RuntimeUiDocument,
    snapshot: &UiStateSnapshot,
    active_theme: Option<&UiTheme>,
) -> Option<ResolvedUiOverlayDocument> {
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    let root_path = format!("{entity_name}.{root_segment}");
    let mut click_bindings = BTreeMap::new();
    let mut change_bindings = BTreeMap::new();
    let root = resolve_ui_overlay_node(
        &document.root,
        &root_path,
        snapshot,
        active_theme,
        &mut click_bindings,
        &mut change_bindings,
    )?;

    Some(ResolvedUiOverlayDocument {
        overlay: UiOverlayDocument {
            entity_name: entity_name.to_owned(),
            layer: resolve_ui_overlay_layer(&document.target),
            viewport: resolve_ui_overlay_viewport(&document.target),
            root,
        },
        click_bindings,
        change_bindings,
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

fn resolve_ui_overlay_viewport(target: &RuntimeUiTarget) -> Option<UiOverlayViewport> {
    match target {
        RuntimeUiTarget::ScreenSpace { viewport, .. } => {
            viewport.map(|viewport| UiOverlayViewport {
                width: viewport.width,
                height: viewport.height,
                scaling: match viewport.scaling {
                    RuntimeUiViewportScaling::Expand => UiOverlayViewportScaling::Expand,
                    RuntimeUiViewportScaling::Fixed => UiOverlayViewportScaling::Fixed,
                    RuntimeUiViewportScaling::Fit => UiOverlayViewportScaling::Fit,
                },
            })
        }
    }
}

fn resolve_ui_overlay_node(
    node: &RuntimeUiNode,
    path: &str,
    snapshot: &UiStateSnapshot,
    active_theme: Option<&UiTheme>,
    click_bindings: &mut BTreeMap<String, UiEventBinding>,
    change_bindings: &mut BTreeMap<String, UiEventBinding>,
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
        RuntimeUiNodeKind::Slider {
            value,
            min,
            max,
            step,
        } => UiOverlayNodeKind::Slider {
            value: snapshot
                .value_overrides
                .get(path)
                .copied()
                .unwrap_or_else(|| normalize_slider_value(*value, *min, *max))
                .clamp(0.0, 1.0),
            min: *min,
            max: *max,
            step: *step,
        },
        RuntimeUiNodeKind::Toggle {
            checked,
            text,
            font,
        } => UiOverlayNodeKind::Toggle {
            checked: snapshot
                .value_overrides
                .get(path)
                .map(|value| *value >= 0.5)
                .unwrap_or(*checked),
            text: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| text.clone()),
            font: font.clone(),
        },
        RuntimeUiNodeKind::OptionSet {
            selected,
            options,
            font,
        } => {
            let options = snapshot
                .options_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| options.clone());
            let selected = selected_or_first_option(snapshot, path, selected, &options);
            UiOverlayNodeKind::OptionSet {
                selected,
                options,
                font: font.clone(),
            }
        }
        RuntimeUiNodeKind::Dropdown {
            selected,
            options,
            font,
        } => {
            let options = snapshot
                .options_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| options.clone());
            let selected = selected_or_first_option(snapshot, path, selected, &options);
            UiOverlayNodeKind::Dropdown {
                selected,
                options,
                expanded: snapshot
                    .expanded_overrides
                    .get(path)
                    .copied()
                    .unwrap_or(false),
                font: font.clone(),
            }
        }
        RuntimeUiNodeKind::ColorPickerRgb { color } => UiOverlayNodeKind::ColorPickerRgb {
            color: snapshot
                .background_overrides
                .get(path)
                .copied()
                .unwrap_or(*color),
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
        if let Some(child) = resolve_ui_overlay_node(
            child,
            &child_path,
            snapshot,
            active_theme,
            click_bindings,
            change_bindings,
        ) {
            children.push(child);
        }
    }

    if let Some(binding) = node.events.on_click.as_ref() {
        click_bindings.insert(path.to_owned(), binding.clone());
    }
    if let Some(binding) = node.events.on_change.as_ref() {
        change_bindings.insert(path.to_owned(), binding.clone());
    }

    Some(UiOverlayNode {
        id: node.id.clone(),
        kind,
        style: resolve_ui_overlay_style_with_overrides(
            active_theme,
            node.style_class.as_deref(),
            &node.style,
            path,
            snapshot,
        ),
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
        RuntimeUiNodeKind::Slider { .. } => "slider",
        RuntimeUiNodeKind::Toggle { .. } => "toggle",
        RuntimeUiNodeKind::OptionSet { .. } => "option-set",
        RuntimeUiNodeKind::Dropdown { .. } => "dropdown",
        RuntimeUiNodeKind::ColorPickerRgb { .. } => "color-picker-rgb",
        RuntimeUiNodeKind::Spacer => "spacer",
    }
}

fn selected_or_first_option(
    snapshot: &UiStateSnapshot,
    path: &str,
    selected: &str,
    options: &[String],
) -> String {
    let selected = snapshot
        .selected_overrides
        .get(path)
        .map(String::as_str)
        .unwrap_or(selected);
    if options.iter().any(|option| option == selected) {
        return selected.to_owned();
    }
    options
        .first()
        .cloned()
        .unwrap_or_else(|| selected.to_owned())
}

fn normalize_slider_value(value: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    if range.abs() <= f32::EPSILON {
        return 0.0;
    }
    ((value - min) / range).clamp(0.0, 1.0)
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
        text_anchor: match style.align {
            RuntimeUiTextAlign::Start => UiTextAnchor::TopLeft,
            RuntimeUiTextAlign::Center => UiTextAnchor::Center,
        },
    }
}

fn resolve_ui_overlay_style_with_overrides(
    active_theme: Option<&UiTheme>,
    style_class: Option<&str>,
    style: &RuntimeUiStyle,
    path: &str,
    snapshot: &UiStateSnapshot,
) -> UiOverlayStyle {
    let mut merged = active_theme
        .and_then(|theme| style_class.and_then(|style_class| theme.classes.get(style_class)))
        .cloned()
        .unwrap_or_default();
    merge_ui_style(&mut merged, style);
    let mut style = resolve_ui_overlay_style(&merged);
    if let Some(color) = snapshot.color_overrides.get(path).copied() {
        style.color = Some(color);
    }
    if let Some(background) = snapshot.background_overrides.get(path).copied() {
        style.background = Some(background);
    }
    style
}

fn merge_ui_style(base: &mut RuntimeUiStyle, overlay: &RuntimeUiStyle) {
    base.left = overlay.left.or(base.left);
    base.top = overlay.top.or(base.top);
    base.right = overlay.right.or(base.right);
    base.bottom = overlay.bottom.or(base.bottom);
    base.width = overlay.width.or(base.width);
    base.height = overlay.height.or(base.height);
    if overlay.padding != 0.0 {
        base.padding = overlay.padding;
    }
    if overlay.gap != 0.0 {
        base.gap = overlay.gap;
    }
    base.background = overlay.background.or(base.background);
    base.color = overlay.color.or(base.color);
    base.border_color = overlay.border_color.or(base.border_color);
    if overlay.border_width != 0.0 {
        base.border_width = overlay.border_width;
    }
    if overlay.border_radius != 0.0 {
        base.border_radius = overlay.border_radius;
    }
    if overlay.font_size != RuntimeUiStyle::default().font_size {
        base.font_size = overlay.font_size;
    }
    base.word_wrap = overlay.word_wrap || base.word_wrap;
    base.fit_to_width = overlay.fit_to_width || base.fit_to_width;
    if overlay.align != RuntimeUiTextAlign::Start {
        base.align = overlay.align;
    }
}

pub(crate) fn hit_test_ui_layout(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if let Some(path) = hit_test_expanded_dropdown(node, x, y) {
        return Some(path);
    }

    hit_test_ui_layout_normal(node, x, y)
}

fn hit_test_ui_layout_normal(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if x < node.rect.x
        || y < node.rect.y
        || x > node.rect.x + node.rect.width
        || y > node.rect.y + node.rect.height
    {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_ui_layout_normal(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}

fn hit_test_expanded_dropdown(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_expanded_dropdown(child, x, y) {
            return Some(path);
        }
    }

    let UiOverlayNodeKind::Dropdown {
        expanded: true,
        options,
        ..
    } = &node.node.kind
    else {
        return None;
    };

    let row_height = 38.0_f32.min(node.rect.height.max(0.0));
    let total_height = row_height * (options.len() as f32 + 1.0);
    if x >= node.rect.x
        && x <= node.rect.x + node.rect.width
        && y >= node.rect.y
        && y <= node.rect.y + total_height
    {
        return Some(node.path.clone());
    }

    None
}

pub(crate) fn find_ui_layout_node<'a>(
    node: &'a OverlayUiLayoutNode,
    path: &str,
) -> Option<&'a OverlayUiLayoutNode> {
    if node.path == path {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_ui_layout_node(child, path) {
            return Some(found);
        }
    }
    None
}
