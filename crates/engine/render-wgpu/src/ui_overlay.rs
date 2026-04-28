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
    pub root: UiOverlayNode,
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
    for document in &ordered {
        let layout = build_ui_layout_tree(viewport, document);
        append_layout_primitives(&layout, &mut primitives);
    }

    primitives
}

pub fn build_ui_layout_tree(
    viewport: UiViewportSize,
    document: &UiOverlayDocument,
) -> UiLayoutNode {
    let measured = measure_node(&document.root);
    let width = document.root.style.width.unwrap_or(measured.x).max(0.0);
    let height = document.root.style.height.unwrap_or(measured.y).max(0.0);
    let x = resolve_screen_axis(
        document.root.style.left,
        document.root.style.right,
        viewport.width,
        width,
    );
    let y = resolve_screen_axis(
        document.root.style.top,
        document.root.style.bottom,
        viewport.height,
        height,
    );
    let root_rect = UiRect::new(x, y, width, height);
    layout_node(
        &document.entity_name,
        &document.root,
        root_rect,
        document
            .root
            .id
            .clone()
            .unwrap_or_else(|| "root".to_owned()),
        0,
    )
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
            entity_name,
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
        let height = child.style.height.unwrap_or(measured.y).max(0.0);
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
        let height = child
            .style
            .height
            .unwrap_or_else(|| default_child_height_for_row(child, content.height, measured.y))
            .max(0.0);
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

fn measure_node(node: &UiOverlayNode) -> Vec2 {
    let padding = node.style.padding.max(0.0);
    let gap = node.style.gap.max(0.0);
    let intrinsic = match &node.kind {
        UiOverlayNodeKind::Text { content, .. } => measure_text(content, node.style.font_size),
        UiOverlayNodeKind::Button { text, .. } => {
            let label = measure_text(text, node.style.font_size.max(16.0));
            Vec2::new(
                label.x + padding * 2.0 + 24.0,
                label.y + padding * 2.0 + 12.0,
            )
        }
        UiOverlayNodeKind::ProgressBar { .. } => Vec2::new(220.0, 18.0),
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

    Vec2::new(
        node.style.width.unwrap_or(intrinsic.x).max(0.0),
        node.style.height.unwrap_or(intrinsic.y).max(0.0),
    )
}

fn measure_text(content: &str, font_size: f32) -> Vec2 {
    let effective_font_size = font_size.max(8.0);
    let advance = effective_font_size * (6.0 / 7.0);
    let line_height = effective_font_size * 1.2;
    let mut max_width: f32 = 0.0;
    let mut line_count = 0usize;

    for line in content.split('\n') {
        max_width = max_width.max(line.chars().count() as f32 * advance);
        line_count += 1;
    }

    if line_count == 0 {
        line_count = 1;
    }

    Vec2::new(max_width, line_count as f32 * line_height)
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
            anchor: UiTextAnchor::TopLeft,
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

fn default_child_width_for_column(
    node: &UiOverlayNode,
    content_width: f32,
    measured_width: f32,
) -> f32 {
    match node.kind {
        UiOverlayNodeKind::Panel
        | UiOverlayNodeKind::Column
        | UiOverlayNodeKind::Row
        | UiOverlayNodeKind::Stack
        | UiOverlayNodeKind::ProgressBar { .. }
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
        | UiOverlayNodeKind::ProgressBar { .. } => measured_height,
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
        UiOverlayStyle, UiTextAnchor, UiViewportSize, build_ui_layout_tree,
        build_ui_overlay_primitives,
    };
    use amigo_math::ColorRgba;

    #[test]
    fn computes_column_layout() {
        let document = UiOverlayDocument {
            entity_name: "playground-2d-ui-preview".to_owned(),
            layer: UiOverlayLayer::Hud,
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
}
