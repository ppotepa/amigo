use amigo_math::Vec2;

use crate::model::{LayoutElement, LayoutKind, LayoutLeafKind, LayoutTab};

pub(crate) fn measure_element<T>(node: &LayoutElement<T>) -> (f32, f32) {
    let padding = node.style.padding.max(0.0);
    let gap = node.style.gap.max(0.0);

    let intrinsic = match &node.kind {
        LayoutKind::Leaf(leaf) => match leaf {
            LayoutLeafKind::Text { content } => measure_text_block(
                content,
                node.style.width.unwrap_or(0.0),
                node.style.font_size,
                node.style.word_wrap,
                node.style.fit_to_width,
            ),
            LayoutLeafKind::Button { text } => {
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
            LayoutLeafKind::ProgressBar => Vec2::new(220.0, 18.0),
            LayoutLeafKind::Slider => Vec2::new(220.0, 24.0),
            LayoutLeafKind::Toggle { text } => {
                let label = measure_text_block(
                    text,
                    node.style.width.unwrap_or(0.0),
                    node.style.font_size.max(14.0),
                    node.style.word_wrap,
                    node.style.fit_to_width,
                );
                Vec2::new(label.x + 64.0, label.y.max(22.0) + padding * 2.0)
            }
            LayoutLeafKind::OptionSet { option_count } => {
                Vec2::new((*option_count).max(1) as f32 * 108.0, 38.0)
            }
            LayoutLeafKind::Dropdown { .. } => Vec2::new(220.0, 38.0),
            LayoutLeafKind::ColorPickerRgb => Vec2::new(260.0, 118.0),
            LayoutLeafKind::CurveEditor => Vec2::new(260.0, 118.0),
            LayoutLeafKind::Spacer => Vec2::new(0.0, 0.0),
        },
        LayoutKind::GroupBox { .. } => {
            let children = measure_column_like_children(&node.children, padding, gap);
            Vec2::new(
                children.x,
                children.y + group_box_label_height(node.style.font_size),
            )
        }
        LayoutKind::TabView { selected, tabs } => {
            let selected = selected_tab_id(selected, tabs, &node.children);
            let panel = node
                .children
                .iter()
                .find(|child| child.id.as_deref() == Some(selected.as_str()))
                .map(measure_element)
                .map(|(x, y)| Vec2::new(x, y))
                .unwrap_or(Vec2::new(0.0, 0.0));
            Vec2::new(
                panel.x.max((tabs.len().max(1) as f32) * 108.0) + padding * 2.0,
                panel.y + tab_view_header_height(node.style.font_size, padding) + padding * 2.0,
            )
        }
        LayoutKind::Row => {
            let mut width = 0.0;
            let mut height: f32 = 0.0;
            for (index, child) in node.children.iter().enumerate() {
                let (cw, ch) = measure_element(child);
                width += cw;
                if index > 0 {
                    width += gap;
                }
                height = height.max(ch);
            }
            Vec2::new(width + padding * 2.0, height + padding * 2.0)
        }
        LayoutKind::Column | LayoutKind::Panel => {
            measure_column_like_children(&node.children, padding, gap)
        }
        LayoutKind::Stack => {
            let mut width: f32 = 0.0;
            let mut height: f32 = 0.0;
            for child in &node.children {
                let (cw, ch) = measure_element(child);
                width = width.max(cw);
                height = height.max(ch);
            }
            Vec2::new(width + padding * 2.0, height + padding * 2.0)
        }
    };

    (
        node.style.width.unwrap_or(intrinsic.x).max(0.0),
        node.style.height.unwrap_or(intrinsic.y).max(0.0),
    )
}

pub(crate) fn group_box_label_height(font_size: f32) -> f32 {
    font_size.max(8.0) * 1.2
}

pub(crate) fn tab_view_header_height(font_size: f32, padding: f32) -> f32 {
    (font_size.max(14.0) * 1.2 + padding * 2.0).max(38.0)
}

pub(crate) fn selected_tab_id<T>(
    selected: &str,
    tabs: &[LayoutTab],
    children: &[LayoutElement<T>],
) -> String {
    if tabs.iter().any(|tab| tab.id == selected) {
        return selected.to_owned();
    }
    tabs.first()
        .map(|tab| tab.id.clone())
        .or_else(|| children.iter().find_map(|child| child.id.clone()))
        .unwrap_or_default()
}

fn measure_column_like_children<T>(children: &[LayoutElement<T>], padding: f32, gap: f32) -> Vec2 {
    let mut width: f32 = 0.0;
    let mut height = 0.0;
    for (index, child) in children.iter().enumerate() {
        let (cw, ch) = measure_element(child);
        width = width.max(cw);
        height += ch;
        if index > 0 {
            height += gap;
        }
    }
    Vec2::new(width + padding * 2.0, height + padding * 2.0)
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

fn layout_text_lines(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> (f32, Vec<String>) {
    let mut effective_font_size = font_size.max(8.0);
    if fit_to_width && !word_wrap && max_width > 0.0 {
        let width = measure_text_line_width(content, effective_font_size);
        if width > max_width {
            effective_font_size = (effective_font_size * (max_width / width))
                .max(8.0)
                .min(effective_font_size);
        }
    }

    let lines = if word_wrap && max_width > 0.0 {
        wrap_text_lines(content, effective_font_size, max_width)
    } else {
        content.split('\n').map(|line| line.to_owned()).collect()
    };

    (
        effective_font_size,
        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        },
    )
}

fn wrap_text_lines(content: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
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
                lines.push(current.clone());
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
                    lines.push(fragment.clone());
                    fragment.clear();
                }
                fragment.push(ch);
            }
            current = fragment;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn measure_text_line_width(content: &str, font_size: f32) -> f32 {
    let effective_font_size = font_size.max(8.0);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    content.chars().count() as f32 * advance
}
