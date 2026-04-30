use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiOverlayLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiViewportSize {
    pub width: f32,
    pub height: f32,
}

impl UiViewportSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayDocument {
    pub entity_name: String,
    pub layer: UiOverlayLayer,
    pub viewport: Option<UiOverlayViewport>,
    pub root: UiOverlayNode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiOverlayViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: UiOverlayViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiOverlayViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayNode {
    pub id: Option<String>,
    pub kind: UiOverlayNodeKind,
    pub style: UiOverlayStyle,
    pub children: Vec<UiOverlayNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiOverlayNodeKind {
    Panel,
    Row,
    Column,
    Stack,
    Text {
        content: String,
        font: Option<AssetKey>,
    },
    Button {
        text: String,
        font: Option<AssetKey>,
    },
    ProgressBar {
        value: f32,
    },
    Slider {
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Toggle {
        checked: bool,
        text: String,
        font: Option<AssetKey>,
    },
    OptionSet {
        selected: String,
        options: Vec<String>,
        font: Option<AssetKey>,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
        expanded: bool,
        font: Option<AssetKey>,
    },
    ColorPickerRgb {
        color: ColorRgba,
    },
    Spacer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayStyle {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub gap: f32,
    pub background: Option<ColorRgba>,
    pub color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub word_wrap: bool,
    pub fit_to_width: bool,
    pub text_anchor: UiTextAnchor,
}

impl Default for UiOverlayStyle {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            padding: 0.0,
            gap: 0.0,
            background: None,
            color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            font_size: 16.0,
            word_wrap: false,
            fit_to_width: false,
            text_anchor: UiTextAnchor::TopLeft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn inset(self, inset: f32) -> Self {
        let clamped = inset.max(0.0).min(self.width * 0.5).min(self.height * 0.5);
        Self {
            x: self.x + clamped,
            y: self.y + clamped,
            width: (self.width - clamped * 2.0).max(0.0),
            height: (self.height - clamped * 2.0).max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLayoutNode {
    pub path: String,
    pub rect: UiRect,
    pub node: UiOverlayNode,
    pub children: Vec<UiLayoutNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextAnchor {
    TopLeft,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiDrawPrimitive {
    Quad {
        rect: UiRect,
        color: ColorRgba,
    },
    Text {
        rect: UiRect,
        content: String,
        color: ColorRgba,
        font_size: f32,
        font: Option<AssetKey>,
        anchor: UiTextAnchor,
        word_wrap: bool,
        fit_to_width: bool,
    },
    ProgressBar {
        rect: UiRect,
        value: f32,
        background: ColorRgba,
        foreground: ColorRgba,
    },
}

pub fn build_ui_overlay_primitives(
    viewport: UiViewportSize,
    documents: &[UiOverlayDocument],
) -> Vec<UiDrawPrimitive> {
    let mut ordered = documents.to_vec();
    ordered.sort_by_key(|document| document.layer);

    let mut primitives = Vec::new();
    let mut popup_primitives = Vec::new();
    for document in &ordered {
        let layout = build_ui_layout_tree(viewport, document);
        append_layout_primitives(&layout, &mut primitives);
        append_layout_popup_primitives(&layout, &mut popup_primitives);
    }
    primitives.extend(popup_primitives);

    primitives
}

pub fn build_ui_layout_tree(
    viewport: UiViewportSize,
    document: &UiOverlayDocument,
) -> UiLayoutNode {
    let layout_viewport = match document.viewport {
        Some(UiOverlayViewport {
            width,
            height,
            scaling: UiOverlayViewportScaling::Fixed | UiOverlayViewportScaling::Fit,
        }) => UiViewportSize::new(width.max(1.0), height.max(1.0)),
        Some(UiOverlayViewport {
            scaling: UiOverlayViewportScaling::Expand,
            ..
        })
        | None => viewport,
    };
    let measured = measure_node(&document.root);
    let width = document.root.style.width.unwrap_or(measured.x).max(0.0);
    let height = document.root.style.height.unwrap_or(measured.y).max(0.0);
    let x = resolve_screen_axis(
        document.root.style.left,
        document.root.style.right,
        layout_viewport.width,
        width,
    );
    let y = resolve_screen_axis(
        document.root.style.top,
        document.root.style.bottom,
        layout_viewport.height,
        height,
    );
    let root_rect = UiRect::new(x, y, width, height);
    let layout = layout_node(
        &document.entity_name,
        &document.root,
        root_rect,
        document
            .root
            .id
            .clone()
            .unwrap_or_else(|| "root".to_owned()),
        0,
    );

    transform_layout_for_viewport(layout, viewport, document.viewport)
}

fn transform_layout_for_viewport(
    layout: UiLayoutNode,
    viewport: UiViewportSize,
    document_viewport: Option<UiOverlayViewport>,
) -> UiLayoutNode {
    let Some(document_viewport) = document_viewport else {
        return layout;
    };

    if document_viewport.scaling == UiOverlayViewportScaling::Expand {
        return layout;
    }

    let design_width = document_viewport.width.max(1.0);
    let design_height = document_viewport.height.max(1.0);
    let scale = match document_viewport.scaling {
        UiOverlayViewportScaling::Expand => 1.0,
        UiOverlayViewportScaling::Fixed => 1.0,
        UiOverlayViewportScaling::Fit => {
            (viewport.width / design_width).min(viewport.height / design_height)
        }
    }
    .max(0.0);
    let offset = Vec2::new(
        (viewport.width - design_width * scale) * 0.5,
        (viewport.height - design_height * scale) * 0.5,
    );

    transform_layout_node(layout, offset, scale)
}

fn transform_layout_node(node: UiLayoutNode, offset: Vec2, scale: f32) -> UiLayoutNode {
    UiLayoutNode {
        path: node.path,
        rect: UiRect::new(
            offset.x + node.rect.x * scale,
            offset.y + node.rect.y * scale,
            node.rect.width * scale,
            node.rect.height * scale,
        ),
        node: UiOverlayNode {
            id: node.node.id,
            kind: node.node.kind,
            style: scale_overlay_style(node.node.style, scale),
            children: node.node.children,
        },
        children: node
            .children
            .into_iter()
            .map(|child| transform_layout_node(child, offset, scale))
            .collect(),
    }
}

fn scale_overlay_style(mut style: UiOverlayStyle, scale: f32) -> UiOverlayStyle {
    style.padding *= scale;
    style.gap *= scale;
    style.border_width *= scale;
    style.border_radius *= scale;
    style.font_size *= scale;
    style
}

fn layout_node(
    entity_name: &str,
    node: &UiOverlayNode,
    rect: UiRect,
    segment: String,
    depth_index: usize,
) -> UiLayoutNode {
    let path = format!("{entity_name}.{segment}");
    let content = rect.inset(node.style.padding.max(0.0));
    let gap = node.style.gap.max(0.0);
    let children = match node.kind {
        UiOverlayNodeKind::Row => layout_row_children(node, content, gap),
        UiOverlayNodeKind::Stack => layout_stack_children(node, content),
        UiOverlayNodeKind::Column | UiOverlayNodeKind::Panel => {
            layout_column_children(node, content, gap)
        }
        UiOverlayNodeKind::Text { .. }
        | UiOverlayNodeKind::Button { .. }
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. }
        | UiOverlayNodeKind::Spacer => Vec::new(),
    };

    let node_with_children = UiOverlayNode {
        id: node.id.clone(),
        kind: node.kind.clone(),
        style: node.style.clone(),
        children: node.children.clone(),
    };

    let mut layout_children = Vec::with_capacity(children.len());
    for (index, (child, child_rect)) in children.into_iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{depth_index}-{index}", kind_slug(&child.kind)));
        layout_children.push(layout_node(
            &path,
            child,
            child_rect,
            segment,
            depth_index + 1,
        ));
    }

    UiLayoutNode {
        path,
        rect,
        node: node_with_children,
        children: layout_children,
    }
}

fn layout_column_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
    gap: f32,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut cursor = content.y;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child
            .style
            .width
            .unwrap_or_else(|| default_child_width_for_column(child, content.width, measured.x))
            .max(0.0);
        let height = resolved_child_height_for_column(child, measured.y).max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = cursor + child.style.top.unwrap_or(0.0);
        laid_out.push((child, UiRect::new(x, y, width, height)));
        cursor = y + height + gap;
    }
    laid_out
}

fn layout_row_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
    gap: f32,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut cursor = content.x;
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child.style.width.unwrap_or(measured.x).max(0.0);
        let height = resolved_child_height_for_row(child, content.height, measured.y).max(0.0);
        let x = cursor + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((child, UiRect::new(x, y, width, height)));
        cursor = x + width + gap;
    }
    laid_out
}

fn layout_stack_children<'a>(
    node: &'a UiOverlayNode,
    content: UiRect,
) -> Vec<(&'a UiOverlayNode, UiRect)> {
    let mut laid_out = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let measured = measure_node(child);
        let width = child
            .style
            .width
            .unwrap_or(content.width.max(measured.x))
            .max(0.0);
        let height = child
            .style
            .height
            .unwrap_or(content.height.max(measured.y))
            .max(0.0);
        let x = content.x + child.style.left.unwrap_or(0.0);
        let y = content.y + child.style.top.unwrap_or(0.0);
        laid_out.push((
            child,
            UiRect::new(x, y, width.min(content.width), height.min(content.height)),
        ));
    }
    laid_out
}

fn resolved_child_height_for_column(child: &UiOverlayNode, measured_height: f32) -> f32 {
    child.style.height.unwrap_or(measured_height)
}

fn resolved_child_height_for_row(
    child: &UiOverlayNode,
    content_height: f32,
    measured_height: f32,
) -> f32 {
    let default_height = default_child_height_for_row(child, content_height, measured_height);
    child.style.height.unwrap_or(default_height)
}

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
        UiOverlayNodeKind::ColorPickerRgb { .. } => Vec2::new(260.0, 118.0),
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

fn append_layout_primitives(layout: &UiLayoutNode, primitives: &mut Vec<UiDrawPrimitive>) {
    if let Some(background) = layout.node.style.background {
        primitives.push(UiDrawPrimitive::Quad {
            rect: layout.rect,
            color: background,
        });
    }

    if let Some(border_color) = layout.node.style.border_color {
        append_border_primitives(
            primitives,
            layout.rect,
            border_color,
            layout.node.style.border_width.max(0.0),
        );
    }

    match &layout.node.kind {
        UiOverlayNodeKind::Text { content, font } => primitives.push(UiDrawPrimitive::Text {
            rect: layout.rect,
            content: content.clone(),
            color: layout.node.style.color.unwrap_or(ColorRgba::WHITE),
            font_size: layout.node.style.font_size.max(8.0),
            font: font.clone(),
            anchor: layout.node.style.text_anchor,
            word_wrap: layout.node.style.word_wrap,
            fit_to_width: layout.node.style.fit_to_width,
        }),
        UiOverlayNodeKind::Button { text, font } => {
            if layout.node.style.background.is_none() {
                primitives.push(UiDrawPrimitive::Quad {
                    rect: layout.rect,
                    color: ColorRgba::new(0.2, 0.33, 0.66, 1.0),
                });
            }
            primitives.push(UiDrawPrimitive::Text {
                rect: layout
                    .rect
                    .inset(layout.node.style.padding.max(0.0).max(8.0)),
                content: text.clone(),
                color: layout.node.style.color.unwrap_or(ColorRgba::WHITE),
                font_size: layout.node.style.font_size.max(14.0),
                font: font.clone(),
                anchor: UiTextAnchor::Center,
                word_wrap: layout.node.style.word_wrap,
                fit_to_width: layout.node.style.fit_to_width,
            });
        }
        UiOverlayNodeKind::ProgressBar { value } => primitives.push(UiDrawPrimitive::ProgressBar {
            rect: layout.rect,
            value: value.clamp(0.0, 1.0),
            background: layout
                .node
                .style
                .background
                .unwrap_or(ColorRgba::new(0.19, 0.21, 0.29, 1.0)),
            foreground: layout
                .node
                .style
                .color
                .unwrap_or(ColorRgba::new(0.4, 0.8, 0.53, 1.0)),
        }),
        UiOverlayNodeKind::Slider { value, .. } => {
            append_slider_primitives(layout, primitives, value.clamp(0.0, 1.0));
        }
        UiOverlayNodeKind::Toggle {
            checked,
            text,
            font,
        } => {
            append_toggle_primitives(layout, primitives, *checked, text, font);
        }
        UiOverlayNodeKind::OptionSet {
            selected,
            options,
            font,
        } => append_option_set_primitives(layout, primitives, selected, options, font),
        UiOverlayNodeKind::Dropdown { selected, font, .. } => {
            append_dropdown_header_primitives(layout, primitives, selected, font)
        }
        UiOverlayNodeKind::ColorPickerRgb { color } => {
            append_color_picker_rgb_primitives(layout, primitives, *color);
        }
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::Spacer => {}
    }

    for child in &layout.children {
        append_layout_primitives(child, primitives);
    }
}

fn append_layout_popup_primitives(layout: &UiLayoutNode, primitives: &mut Vec<UiDrawPrimitive>) {
    for child in &layout.children {
        append_layout_popup_primitives(child, primitives);
    }

    if let UiOverlayNodeKind::Dropdown {
        selected,
        options,
        expanded: true,
        font,
    } = &layout.node.kind
    {
        append_dropdown_popup_primitives(layout, primitives, selected, options, font);
    }
}

fn append_border_primitives(
    primitives: &mut Vec<UiDrawPrimitive>,
    rect: UiRect,
    color: ColorRgba,
    width: f32,
) {
    if width <= 0.0 {
        return;
    }

    let horizontal = width.min(rect.height * 0.5);
    let vertical = width.min(rect.width * 0.5);
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(rect.x, rect.y, rect.width, horizontal),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(
            rect.x,
            rect.y + rect.height - horizontal,
            rect.width,
            horizontal,
        ),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(rect.x, rect.y, vertical, rect.height),
        color,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(
            rect.x + rect.width - vertical,
            rect.y,
            vertical,
            rect.height,
        ),
        color,
    });
}

fn append_slider_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    value: f32,
) {
    let track = layout.rect.inset(4.0);
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    primitives.push(UiDrawPrimitive::Quad {
        rect: track,
        color: background,
    });
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(track.x, track.y, track.width * value, track.height),
        color: foreground,
    });
    let thumb_width = 10.0_f32.min(layout.rect.width.max(0.0));
    let thumb_x = (track.x + track.width * value - thumb_width * 0.5)
        .max(layout.rect.x)
        .min(layout.rect.x + layout.rect.width - thumb_width);
    primitives.push(UiDrawPrimitive::Quad {
        rect: UiRect::new(thumb_x, layout.rect.y, thumb_width, layout.rect.height),
        color: foreground,
    });
}

fn append_toggle_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    checked: bool,
    text: &str,
    font: &Option<AssetKey>,
) {
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.9, 0.94, 1.0, 1.0));
    let accent = if checked {
        foreground
    } else {
        layout
            .node
            .style
            .border_color
            .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0))
    };
    let switch_width = 42.0_f32.min(layout.rect.width.max(0.0));
    let switch_rect = UiRect::new(
        layout.rect.x,
        layout.rect.y,
        switch_width,
        layout.rect.height,
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: switch_rect.inset(5.0),
        color: layout
            .node
            .style
            .background
            .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0)),
    });
    let knob = if checked {
        UiRect::new(
            switch_rect.x + switch_rect.width - 18.0,
            switch_rect.y + 8.0,
            12.0,
            (switch_rect.height - 16.0).max(0.0),
        )
    } else {
        UiRect::new(
            switch_rect.x + 6.0,
            switch_rect.y + 8.0,
            12.0,
            (switch_rect.height - 16.0).max(0.0),
        )
    };
    primitives.push(UiDrawPrimitive::Quad {
        rect: knob,
        color: accent,
    });
    if !text.is_empty() {
        primitives.push(UiDrawPrimitive::Text {
            rect: UiRect::new(
                layout.rect.x + switch_width + 8.0,
                layout.rect.y,
                (layout.rect.width - switch_width - 8.0).max(0.0),
                layout.rect.height,
            ),
            content: text.to_owned(),
            color: foreground,
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::TopLeft,
            word_wrap: layout.node.style.word_wrap,
            fit_to_width: layout.node.style.fit_to_width,
        });
    }
}

fn append_option_set_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    options: &[String],
    font: &Option<AssetKey>,
) {
    if options.is_empty() {
        return;
    }
    let segment_width = layout.rect.width / options.len() as f32;
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));
    for (index, option) in options.iter().enumerate() {
        let rect = UiRect::new(
            layout.rect.x + index as f32 * segment_width,
            layout.rect.y,
            segment_width,
            layout.rect.height,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect,
            color: if option == selected {
                foreground
            } else {
                background
            },
        });
        append_border_primitives(primitives, rect, border, 1.0);
        primitives.push(UiDrawPrimitive::Text {
            rect: rect.inset(6.0),
            content: option.clone(),
            color: if option == selected {
                background
            } else {
                foreground
            },
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::Center,
            word_wrap: false,
            fit_to_width: true,
        });
    }
}

fn append_dropdown_header_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    font: &Option<AssetKey>,
) {
    let row_height = 38.0_f32.min(layout.rect.height.max(0.0));
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));
    let header = UiRect::new(layout.rect.x, layout.rect.y, layout.rect.width, row_height);
    primitives.push(UiDrawPrimitive::Quad {
        rect: header,
        color: background,
    });
    append_border_primitives(primitives, header, border, 1.0);
    primitives.push(UiDrawPrimitive::Text {
        rect: header.inset(8.0),
        content: format!("{selected} v"),
        color: foreground,
        font_size: layout.node.style.font_size.max(14.0),
        font: font.clone(),
        anchor: UiTextAnchor::TopLeft,
        word_wrap: false,
        fit_to_width: true,
    });
}

fn append_dropdown_popup_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    selected: &str,
    options: &[String],
    font: &Option<AssetKey>,
) {
    let row_height = 38.0_f32.min(layout.rect.height.max(0.0));
    if row_height <= 0.0 {
        return;
    }
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.18, 0.2, 0.27, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.35, 0.78, 0.95, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));
    for (index, option) in options.iter().enumerate() {
        let rect = UiRect::new(
            layout.rect.x,
            layout.rect.y + row_height * (index as f32 + 1.0),
            layout.rect.width,
            row_height,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect,
            color: if option == selected {
                foreground
            } else {
                background
            },
        });
        append_border_primitives(primitives, rect, border, 1.0);
        primitives.push(UiDrawPrimitive::Text {
            rect: rect.inset(8.0),
            content: option.clone(),
            color: if option == selected {
                background
            } else {
                foreground
            },
            font_size: layout.node.style.font_size.max(14.0),
            font: font.clone(),
            anchor: UiTextAnchor::TopLeft,
            word_wrap: false,
            fit_to_width: true,
        });
    }
}

fn append_color_picker_rgb_primitives(
    layout: &UiLayoutNode,
    primitives: &mut Vec<UiDrawPrimitive>,
    color: ColorRgba,
) {
    let background = layout
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.12, 0.14, 0.2, 1.0));
    let foreground = layout
        .node
        .style
        .color
        .unwrap_or(ColorRgba::new(0.88, 0.94, 1.0, 1.0));
    let border = layout
        .node
        .style
        .border_color
        .unwrap_or(ColorRgba::new(0.35, 0.4, 0.48, 1.0));

    primitives.push(UiDrawPrimitive::Quad {
        rect: layout.rect,
        color: background,
    });
    append_border_primitives(primitives, layout.rect, border, 1.0);

    let padding = 8.0;
    let swatch = UiRect::new(
        layout.rect.x + padding,
        layout.rect.y + padding,
        54.0_f32.min((layout.rect.width - padding * 2.0).max(0.0)),
        (layout.rect.height - padding * 2.0).max(0.0),
    );
    primitives.push(UiDrawPrimitive::Quad {
        rect: swatch,
        color,
    });
    append_border_primitives(primitives, swatch, border, 1.0);

    let slider_x = swatch.x + swatch.width + 10.0;
    let slider_width = (layout.rect.x + layout.rect.width - padding - slider_x).max(0.0);
    let slider_height = 22.0;
    for (index, (label, value, channel_color)) in [
        ("R", color.r, ColorRgba::new(0.95, 0.24, 0.28, 1.0)),
        ("G", color.g, ColorRgba::new(0.32, 0.86, 0.42, 1.0)),
        ("B", color.b, ColorRgba::new(0.26, 0.54, 1.0, 1.0)),
    ]
    .into_iter()
    .enumerate()
    {
        let y = layout.rect.y + padding + index as f32 * (slider_height + 10.0);
        let label_rect = UiRect::new(slider_x, y, 18.0, slider_height);
        primitives.push(UiDrawPrimitive::Text {
            rect: label_rect,
            content: label.to_owned(),
            color: foreground,
            font_size: layout.node.style.font_size.max(12.0),
            font: None,
            anchor: UiTextAnchor::Center,
            word_wrap: false,
            fit_to_width: true,
        });
        let track = UiRect::new(
            slider_x + 24.0,
            y + 5.0,
            (slider_width - 24.0).max(0.0),
            12.0,
        );
        primitives.push(UiDrawPrimitive::Quad {
            rect: track,
            color: ColorRgba::new(0.04, 0.05, 0.08, 1.0),
        });
        primitives.push(UiDrawPrimitive::Quad {
            rect: UiRect::new(
                track.x,
                track.y,
                track.width * value.clamp(0.0, 1.0),
                track.height,
            ),
            color: channel_color,
        });
    }
}

fn default_child_width_for_column(
    node: &UiOverlayNode,
    content_width: f32,
    measured_width: f32,
) -> f32 {
    if matches!(
        node.kind,
        UiOverlayNodeKind::Text { .. } | UiOverlayNodeKind::Button { .. }
    ) && (node.style.fit_to_width || node.style.word_wrap)
    {
        return content_width.max(measured_width).max(0.0);
    }

    match node.kind {
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. }
        | UiOverlayNodeKind::Spacer => content_width.max(measured_width),
        UiOverlayNodeKind::Text { .. } | UiOverlayNodeKind::Button { .. } => measured_width,
    }
}

fn default_child_height_for_row(
    node: &UiOverlayNode,
    content_height: f32,
    measured_height: f32,
) -> f32 {
    match node.kind {
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::Spacer => content_height.max(measured_height),
        UiOverlayNodeKind::Text { .. }
        | UiOverlayNodeKind::Button { .. }
        | UiOverlayNodeKind::ProgressBar { .. }
        | UiOverlayNodeKind::Slider { .. }
        | UiOverlayNodeKind::Toggle { .. }
        | UiOverlayNodeKind::OptionSet { .. }
        | UiOverlayNodeKind::Dropdown { .. }
        | UiOverlayNodeKind::ColorPickerRgb { .. } => measured_height,
    }
}

fn kind_slug(kind: &UiOverlayNodeKind) -> &'static str {
    match kind {
        UiOverlayNodeKind::Panel => "panel",
        UiOverlayNodeKind::Row => "row",
        UiOverlayNodeKind::Column => "column",
        UiOverlayNodeKind::Stack => "stack",
        UiOverlayNodeKind::Text { .. } => "text",
        UiOverlayNodeKind::Button { .. } => "button",
        UiOverlayNodeKind::ProgressBar { .. } => "progress-bar",
        UiOverlayNodeKind::Slider { .. } => "slider",
        UiOverlayNodeKind::Toggle { .. } => "toggle",
        UiOverlayNodeKind::OptionSet { .. } => "option-set",
        UiOverlayNodeKind::Dropdown { .. } => "dropdown",
        UiOverlayNodeKind::ColorPickerRgb { .. } => "color-picker-rgb",
        UiOverlayNodeKind::Spacer => "spacer",
    }
}

fn resolve_screen_axis(start: Option<f32>, end: Option<f32>, viewport: f32, size: f32) -> f32 {
    if let Some(start) = start {
        start
    } else if let Some(end) = end {
        viewport - size - end
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiDrawPrimitive, UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind,
        UiOverlayStyle, UiOverlayViewport, UiOverlayViewportScaling, UiTextAnchor, UiViewportSize,
        build_ui_layout_tree, build_ui_overlay_primitives,
    };
    use amigo_math::ColorRgba;

    #[test]
    fn computes_column_layout() {
        let document = UiOverlayDocument {
            entity_name: "playground-2d-ui-preview".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 16.0,
                    gap: 12.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("title".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "AMIGO".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            font_size: 28.0,
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("bar".to_owned()),
                        kind: UiOverlayNodeKind::ProgressBar { value: 0.75 },
                        style: UiOverlayStyle {
                            width: Some(120.0),
                            height: Some(18.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.path, "playground-2d-ui-preview.root");
        assert_eq!(layout.rect.x, 24.0);
        assert_eq!(layout.rect.y, 24.0);
        assert_eq!(layout.children[0].rect.x, 40.0);
        assert_eq!(layout.children[0].rect.y, 40.0);
        assert_eq!(layout.children[1].rect.x, 40.0);
        assert!(layout.children[1].rect.y > layout.children[0].rect.y);
    }

    #[test]
    fn builds_ui_primitives_for_button_and_progress_bar() {
        let document = UiOverlayDocument {
            entity_name: "playground-2d-ui-preview".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(240.0),
                    padding: 12.0,
                    gap: 10.0,
                    background: Some(ColorRgba::new(0.1, 0.12, 0.18, 0.9)),
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("button".to_owned()),
                        kind: UiOverlayNodeKind::Button {
                            text: "Emit".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(160.0),
                            height: Some(36.0),
                            background: Some(ColorRgba::new(0.2, 0.33, 0.66, 1.0)),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("hp".to_owned()),
                        kind: UiOverlayNodeKind::ProgressBar { value: 0.5 },
                        style: UiOverlayStyle {
                            width: Some(180.0),
                            height: Some(18.0),
                            background: Some(ColorRgba::new(0.18, 0.2, 0.27, 1.0)),
                            color: Some(ColorRgba::new(0.4, 0.8, 0.53, 1.0)),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        assert!(primitives.iter().any(|primitive| matches!(
            primitive,
            UiDrawPrimitive::Text {
                content,
                anchor: UiTextAnchor::Center,
                ..
            } if content == "Emit"
        )));
        assert!(primitives.iter().any(|primitive| matches!(
            primitive,
            UiDrawPrimitive::ProgressBar { value, .. } if (*value - 0.5).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn respects_layer_order_for_overlay_documents() {
        let background = UiOverlayDocument {
            entity_name: "background-ui".to_owned(),
            layer: UiOverlayLayer::Background,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Text {
                    content: "BACKGROUND".to_owned(),
                    font: None,
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };
        let debug = UiOverlayDocument {
            entity_name: "debug-ui".to_owned(),
            layer: UiOverlayLayer::Debug,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Text {
                    content: "DEBUG".to_owned(),
                    font: None,
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[debug, background]);
        let first_text = primitives
            .into_iter()
            .find_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            });

        assert_eq!(first_text.as_deref(), Some("BACKGROUND"));
    }

    #[test]
    fn wrapped_text_increases_layout_height() {
        let document = UiOverlayDocument {
            entity_name: "debug-ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(24.0),
                    width: Some(200.0),
                    padding: 12.0,
                    gap: 8.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("debug".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "grounded=false vx=120 vy=-10 anim=run".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(176.0),
                            font_size: 14.0,
                            word_wrap: true,
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("message".to_owned()),
                        kind: UiOverlayNodeKind::Text {
                            content: "READY".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle::default(),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert!(layout.children[0].rect.height > 14.0 * 1.2);
        assert!(
            layout.children[1].rect.y >= layout.children[0].rect.y + layout.children[0].rect.height
        );
    }

    #[test]
    fn fixed_fit_viewport_centers_and_scales_design_layout() {
        let document = UiOverlayDocument {
            entity_name: "fixed-ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: Some(UiOverlayViewport {
                width: 1440.0,
                height: 900.0,
                scaling: UiOverlayViewportScaling::Fit,
            }),
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Panel,
                style: UiOverlayStyle {
                    left: Some(24.0),
                    top: Some(18.0),
                    width: Some(1392.0),
                    height: Some(72.0),
                    font_size: 20.0,
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1920.0, 1080.0), &document);

        assert!((layout.rect.x - 124.8).abs() < 0.001);
        assert!((layout.rect.y - 21.6).abs() < 0.001);
        assert!((layout.rect.width - 1670.4).abs() < 0.001);
        assert!((layout.rect.height - 86.4).abs() < 0.001);
    }

    #[test]
    fn expanded_dropdown_does_not_push_sibling_layout() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::Column,
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    width: Some(260.0),
                    gap: 8.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    UiOverlayNode {
                        id: Some("dropdown".to_owned()),
                        kind: UiOverlayNodeKind::Dropdown {
                            selected: "A".to_owned(),
                            options: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                            expanded: true,
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(220.0),
                            height: Some(38.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                    UiOverlayNode {
                        id: Some("button".to_owned()),
                        kind: UiOverlayNodeKind::Button {
                            text: "Below".to_owned(),
                            font: None,
                        },
                        style: UiOverlayStyle {
                            width: Some(220.0),
                            height: Some(40.0),
                            ..UiOverlayStyle::default()
                        },
                        children: Vec::new(),
                    },
                ],
            },
        };

        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);
        assert_eq!(layout.children[0].rect.height, 38.0);
        assert_eq!(layout.children[1].rect.y, 46.0);

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        let text_order = primitives
            .into_iter()
            .filter_map(|primitive| match primitive {
                UiDrawPrimitive::Text { content, .. } => Some(content),
                _ => None,
            })
            .collect::<Vec<_>>();
        let below = text_order
            .iter()
            .position(|content| content == "Below")
            .expect("button text should render");
        let popup_option = text_order
            .iter()
            .rposition(|content| content == "A")
            .expect("dropdown popup option should render");
        assert!(
            popup_option > below,
            "dropdown popup should render after normal sibling primitives"
        );
    }

    #[test]
    fn color_picker_rgb_builds_channel_primitives() {
        let document = UiOverlayDocument {
            entity_name: "ui".to_owned(),
            layer: UiOverlayLayer::Hud,
            viewport: None,
            root: UiOverlayNode {
                id: Some("root".to_owned()),
                kind: UiOverlayNodeKind::ColorPickerRgb {
                    color: ColorRgba::new(0.25, 0.5, 0.75, 1.0),
                },
                style: UiOverlayStyle {
                    left: Some(0.0),
                    top: Some(0.0),
                    width: Some(260.0),
                    height: Some(118.0),
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            },
        };

        let primitives =
            build_ui_overlay_primitives(UiViewportSize::new(1280.0, 720.0), &[document]);
        for label in ["R", "G", "B"] {
            assert!(primitives.iter().any(|primitive| matches!(
                primitive,
                UiDrawPrimitive::Text { content, .. } if content == label
            )));
        }
    }
}
