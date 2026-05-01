fn measure_node(node: &UiOverlayNode) -> Vec2 {
    let padding = node.style.padding.max(0.0);
    let gap = node.style.gap.max(0.0);
    let intrinsic = match &node.kind {
        UiOverlayNodeKind::Text { content, .. } => measure_text_block(
            content,
            node.style.width.unwrap_or(0.0),
            node.style.font_size,
            node.style.word_wrap,
            node.style.fit_to_width,
        ),
        UiOverlayNodeKind::Button { text, .. } => {
            let label = measure_text_block(
                text,
                node.style.width.unwrap_or(0.0),
                node.style.font_size.max(16.0),
                node.style.word_wrap,
                node.style.fit_to_width,
            );
            Vec2::new(
                label.x + padding * 2.0 + 24.0,
                label.y + padding * 2.0 + 12.0,
            )
        }
        UiOverlayNodeKind::GroupBox { .. } => {
            let children = measure_column_like_children(node, padding, gap);
            Vec2::new(children.x, children.y + group_box_label_height(node))
        }
        UiOverlayNodeKind::ProgressBar { .. } => Vec2::new(220.0, 18.0),
        UiOverlayNodeKind::Slider { .. } => Vec2::new(220.0, 24.0),
        UiOverlayNodeKind::Toggle { text, .. } => {
            let label = measure_text_block(
                text,
                node.style.width.unwrap_or(0.0),
                node.style.font_size.max(14.0),
                node.style.word_wrap,
                node.style.fit_to_width,
            );
            Vec2::new(label.x + 64.0, label.y.max(22.0) + padding * 2.0)
        }
        UiOverlayNodeKind::OptionSet { options, .. } => {
            Vec2::new((options.len().max(1) as f32) * 108.0, 38.0)
        }
        UiOverlayNodeKind::Dropdown { .. } => Vec2::new(220.0, 38.0),
        UiOverlayNodeKind::TabView { selected, tabs, .. } => {
            let selected = selected_tab_id(selected, tabs, &node.children);
            let panel = node
                .children
                .iter()
                .find(|child| child.id.as_deref() == Some(selected.as_str()))
                .map(measure_node)
                .unwrap_or(Vec2::new(0.0, 0.0));
            Vec2::new(
                panel.x.max((tabs.len().max(1) as f32) * 108.0) + padding * 2.0,
                panel.y + tab_view_header_height(node) + padding * 2.0,
            )
        }
        UiOverlayNodeKind::ColorPickerRgb { .. } => Vec2::new(260.0, 118.0),
        UiOverlayNodeKind::CurveEditor { .. } => Vec2::new(260.0, 118.0),
        UiOverlayNodeKind::Spacer => Vec2::new(0.0, 0.0),
        UiOverlayNodeKind::Row => {
            let mut width = 0.0;
            let mut height: f32 = 0.0;
            for (index, child) in node.children.iter().enumerate() {
                let size = measure_node(child);
                width += size.x;
                if index > 0 {
                    width += gap;
                }
                height = height.max(size.y);
            }
            Vec2::new(width + padding * 2.0, height + padding * 2.0)
        }
        UiOverlayNodeKind::Column | UiOverlayNodeKind::Panel => {
            measure_column_like_children(node, padding, gap)
        }
        UiOverlayNodeKind::Stack => {
            let mut width: f32 = 0.0;
            let mut height: f32 = 0.0;
            for child in &node.children {
                let size = measure_node(child);
                width = width.max(size.x);
                height = height.max(size.y);
            }
            Vec2::new(width + padding * 2.0, height + padding * 2.0)
        }
    };

    let height = node.style.height.unwrap_or(intrinsic.y);

    Vec2::new(
        node.style.width.unwrap_or(intrinsic.x).max(0.0),
        height.max(0.0),
    )
}

fn measure_text_block(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> Vec2 {
    let (effective_font_size, lines) =
        layout_text_lines(content, max_width, font_size, word_wrap, fit_to_width);
    let line_height = effective_font_size.max(8.0) * 1.2;
    let max_line_width = lines
        .iter()
        .map(|line| measure_text_line_width(line, effective_font_size))
        .fold(0.0, f32::max);

    Vec2::new(max_line_width, (lines.len().max(1) as f32) * line_height)
}

fn measure_column_like_children(node: &UiOverlayNode, padding: f32, gap: f32) -> Vec2 {
    let mut width: f32 = 0.0;
    let mut height = 0.0;
    for (index, child) in node.children.iter().enumerate() {
        let size = measure_node(child);
        width = width.max(size.x);
        height += size.y;
        if index > 0 {
            height += gap;
        }
    }
    Vec2::new(width + padding * 2.0, height + padding * 2.0)
}

pub(crate) fn group_box_label_height(node: &UiOverlayNode) -> f32 {
    node.style.font_size.max(8.0) * 1.2
}

pub(crate) fn tab_view_header_height(node: &UiOverlayNode) -> f32 {
    (node.style.font_size.max(14.0) * 1.2 + node.style.padding.max(0.0) * 2.0).max(38.0)
}

pub fn tab_view_tab_from_mouse(
    rect: UiRect,
    node: &UiOverlayNode,
    tabs: &[UiOverlayTab],
    mouse_x: f32,
    mouse_y: f32,
) -> Option<String> {
    if tabs.is_empty() || mouse_y < rect.y || mouse_y > rect.y + tab_view_header_height(node) {
        return None;
    }
    let tab_width = rect.width / tabs.len() as f32;
    if tab_width <= f32::EPSILON || mouse_x < rect.x || mouse_x > rect.x + rect.width {
        return None;
    }
    let index = (((mouse_x - rect.x) / tab_width).clamp(0.0, 0.999_999) * tabs.len() as f32).floor()
        as usize;
    tabs.get(index).map(|tab| tab.id.clone())
}

pub(crate) fn selected_tab_id(selected: &str, tabs: &[UiOverlayTab], children: &[UiOverlayNode]) -> String {
    if tabs.iter().any(|tab| tab.id == selected) {
        return selected.to_owned();
    }
    tabs.first()
        .map(|tab| tab.id.clone())
        .or_else(|| children.iter().find_map(|child| child.id.clone()))
        .unwrap_or_default()
}

