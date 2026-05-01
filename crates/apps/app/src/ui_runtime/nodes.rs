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
        RuntimeUiNodeKind::GroupBox { label, font } => UiOverlayNodeKind::GroupBox {
            label: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| label.clone()),
            font: font.clone(),
        },
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
            let scroll_offset = snapshot
                .dropdown_scroll_offsets
                .get(path)
                .copied()
                .unwrap_or(0.0)
                .clamp(
                    0.0,
                    options
                        .len()
                        .saturating_sub(dropdown_visible_option_count(options.len()))
                        as f32,
                );
            UiOverlayNodeKind::Dropdown {
                selected,
                options,
                expanded: snapshot
                    .expanded_overrides
                    .get(path)
                    .copied()
                    .unwrap_or(false),
                scroll_offset,
                font: font.clone(),
            }
        }
        RuntimeUiNodeKind::TabView {
            selected,
            tabs,
            font,
        } => UiOverlayNodeKind::TabView {
            selected: selected_or_first_tab(snapshot, path, selected, tabs),
            tabs: tabs
                .iter()
                .map(|tab| amigo_render_wgpu::UiOverlayTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
            font: font.clone(),
        },
        RuntimeUiNodeKind::ColorPickerRgb { color } => UiOverlayNodeKind::ColorPickerRgb {
            color: snapshot
                .background_overrides
                .get(path)
                .copied()
                .unwrap_or(*color),
        },
        RuntimeUiNodeKind::CurveEditor { points } => UiOverlayNodeKind::CurveEditor {
            points: snapshot
                .curve_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| points.clone())
                .into_iter()
                .map(|point| amigo_render_wgpu::UiOverlayCurvePoint {
                    t: point.t,
                    value: point.value,
                })
                .collect(),
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
        RuntimeUiNodeKind::GroupBox { .. } => "group-box",
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
        RuntimeUiNodeKind::TabView { .. } => "tab-view",
        RuntimeUiNodeKind::ColorPickerRgb { .. } => "color-picker-rgb",
        RuntimeUiNodeKind::CurveEditor { .. } => "curve-editor",
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

fn selected_or_first_tab(
    snapshot: &UiStateSnapshot,
    path: &str,
    selected: &str,
    tabs: &[RuntimeUiTab],
) -> String {
    let selected = snapshot
        .selected_overrides
        .get(path)
        .map(String::as_str)
        .unwrap_or(selected);
    if tabs.iter().any(|tab| tab.id == selected) {
        return selected.to_owned();
    }
    tabs.first()
        .map(|tab| tab.id.clone())
        .unwrap_or_else(|| selected.to_owned())
}

fn normalize_slider_value(value: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    if range.abs() <= f32::EPSILON {
        return 0.0;
    }
    ((value - min) / range).clamp(0.0, 1.0)
}

