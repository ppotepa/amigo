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
        UiNodeKind::CurveEditor { .. } => 96.0,
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
