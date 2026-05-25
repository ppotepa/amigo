use amigo_assets::AssetKey;
use amigo_math::ColorRgba;
use amigo_overlay_api::{
    UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle,
    UiOverlayViewport, UiOverlayViewportScaling, UiViewportSize,
};
use amigo_scripting_api::{DevConsoleInputSnapshot, DevConsoleOutputLine, DevConsoleState};

use crate::dev_console_theme::DevConsoleTheme;
use crate::ConsoleCompletionSnapshot;

pub trait DevConsoleOverlayRenderOutput {
    fn push_dev_console_overlay_document(&mut self, document: UiOverlayDocument);
}

pub struct DevConsoleOverlayRenderExtractor;

impl DevConsoleOverlayRenderExtractor {
    pub fn name(&self) -> &'static str {
        "dev_console_overlay"
    }

    pub fn extract(
        &self,
        console: &DevConsoleState,
        completion: Option<&ConsoleCompletionSnapshot>,
        viewport: Option<UiViewportSize>,
        output: &mut impl DevConsoleOverlayRenderOutput,
    ) {
        if let Some(document) = build_dev_console_overlay(console, completion, viewport) {
            output.push_dev_console_overlay_document(document);
        }
    }
}

pub fn build_dev_console_overlay(
    console: &DevConsoleState,
    completion: Option<&ConsoleCompletionSnapshot>,
    viewport: Option<UiViewportSize>,
) -> Option<UiOverlayDocument> {
    build_dev_console_overlay_with_theme(console, completion, viewport, &DevConsoleTheme::default())
}

pub fn build_dev_console_overlay_with_theme(
    console: &DevConsoleState,
    completion: Option<&ConsoleCompletionSnapshot>,
    viewport: Option<UiViewportSize>,
    theme: &DevConsoleTheme,
) -> Option<UiOverlayDocument> {
    if !console.is_open() {
        return None;
    }

    let viewport = viewport.unwrap_or_else(|| {
        UiViewportSize::new(
            theme.viewport.default_width,
            theme.viewport.default_height,
        )
    });
    let layout = &theme.layout;
    let panel_width = (viewport.width - layout.margin * 2.0).max(0.0);
    let panel_height = (viewport.height * layout.panel_height_fraction)
        .clamp(layout.min_panel_height, layout.max_panel_height);
    let panel_left = layout.margin;
    let panel_top = (viewport.height - panel_height - layout.margin).max(layout.margin);
    let content_width =
        (panel_width - layout.panel_padding * 2.0 - layout.scrollbar_width - layout.scrollbar_gap)
            .max(120.0);
    let output_height = (panel_height
        - layout.panel_padding * 2.0
        - layout.header_height
        - layout.input_height
        - layout.output_gap)
        .max(layout.line_height);
    let visible_lines = (output_height / layout.line_height).floor().max(1.0) as usize;
    let output = console.output_window(visible_lines);
    let total_entries = console.output_entries().len();
    let scroll_offset = console.output_scroll_offset();
    let input = console.input_snapshot();

    Some(UiOverlayDocument {
        entity_name: "dev-console-overlay".to_owned(),
        layer: UiOverlayLayer::Debug,
        viewport: Some(UiOverlayViewport {
            width: viewport.width,
            height: viewport.height,
            scaling: UiOverlayViewportScaling::Expand,
        }),
        root: UiOverlayNode {
            id: Some("dev-console-root".to_owned()),
            kind: UiOverlayNodeKind::Stack,
            style: UiOverlayStyle {
                width: Some(viewport.width),
                height: Some(viewport.height),
                ..UiOverlayStyle::default()
            },
            children: {
                let mut children = vec![
                    backdrop_node(viewport, theme),
                    console_panel_node(
                        DevConsolePanelGeometry {
                            left: panel_left,
                            top: panel_top,
                            width: panel_width,
                            height: panel_height,
                            content_width,
                        },
                        DevConsolePanelContent {
                            output,
                            total_entries,
                            visible_lines,
                            scroll_offset,
                            input,
                        },
                        theme,
                    ),
                ];
                if let Some(completion) = completion.filter(|snapshot| snapshot.is_active()) {
                    let popup_height = completion_popup_height(completion, layout.line_height);
                    let popup_left = panel_left + layout.panel_padding;
                    let popup_top = panel_top + panel_height
                        - layout.panel_padding
                        - layout.input_height
                        - popup_height
                        - 8.0;
                    children.push(panel_node(
                        "dev-console-completion-shadow",
                        popup_left + 3.0,
                        popup_top.max(0.0) + 3.0,
                        content_width,
                        popup_height,
                        ColorRgba::new(0.0, 0.0, 0.0, 0.55),
                    ));
                    children.push(completion_popup_node(
                        completion,
                        popup_left,
                        popup_top,
                        content_width,
                        layout.line_height,
                        theme,
                    ));
                }
                children
            },
        },
    })
}

fn backdrop_node(viewport: UiViewportSize, theme: &DevConsoleTheme) -> UiOverlayNode {
    UiOverlayNode {
        id: Some("dev-console-backdrop".to_owned()),
        kind: UiOverlayNodeKind::Panel,
        style: UiOverlayStyle {
            left: Some(0.0),
            top: Some(0.0),
            width: Some(viewport.width),
            height: Some(viewport.height),
            background: Some(theme.colors.backdrop),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

struct DevConsolePanelGeometry {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    content_width: f32,
}

struct DevConsolePanelContent {
    output: Vec<DevConsoleOutputLine>,
    total_entries: usize,
    visible_lines: usize,
    scroll_offset: usize,
    input: DevConsoleInputSnapshot,
}

fn console_panel_node(
    geometry: DevConsolePanelGeometry,
    content: DevConsolePanelContent,
    theme: &DevConsoleTheme,
) -> UiOverlayNode {
    let DevConsolePanelGeometry {
        left,
        top,
        width,
        height,
        content_width,
    } = geometry;
    let DevConsolePanelContent {
        output,
        total_entries,
        visible_lines,
        scroll_offset,
        input,
    } = content;
    let layout = &theme.layout;
    let mut children = Vec::new();
    children.push(text_node(
        "dev-console-header",
        format!(
            "AMIGO DEV CONSOLE  F1 console  F2 reload  Ctrl+R reload  Ctrl+D diagnostics  lines={} scroll={}",
            total_entries, scroll_offset
        ),
        0.0,
        0.0,
        content_width,
        layout.header_height,
        theme.font.header_size,
        theme.colors.header_text,
        theme.text_font(),
    ));

    let output_top = layout.header_height + 6.0;
    for (index, entry) in output.into_iter().enumerate() {
        children.push(text_node(
            format!("dev-console-output-{index}"),
            entry.text,
            0.0,
            output_top + index as f32 * layout.line_height,
            content_width,
            layout.line_height,
            theme.font.output_size,
            theme.level_color(entry.level),
            theme.text_font(),
        ));
    }

    children.extend(scrollbar_nodes(
        width - layout.panel_padding * 2.0 - layout.scrollbar_width,
        output_top,
        layout.scrollbar_width,
        (height
            - layout.panel_padding * 2.0
            - layout.header_height
            - layout.input_height
            - layout.output_gap)
            .max(layout.line_height),
        total_entries,
        visible_lines,
        scroll_offset,
        theme,
    ));

    let input_top = height - layout.panel_padding * 2.0 - layout.input_height;
    children.extend(input_line_nodes(
        &input,
        DevConsoleInputLineLayout {
            left: 0.0,
            top: input_top,
            width: content_width,
            height: layout.input_height,
            font_size: theme.font.input_size,
            color: theme.colors.input_text,
        },
        theme,
    ));

    UiOverlayNode {
        id: Some("dev-console-panel".to_owned()),
        kind: UiOverlayNodeKind::Stack,
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            padding: layout.panel_padding,
            background: Some(theme.colors.panel_background),
            border_color: Some(theme.colors.panel_border),
            border_width: layout.border_width,
            border_radius: layout.border_radius,
            ..UiOverlayStyle::default()
        },
        children,
    }
}

const DEV_CONSOLE_INPUT_PROMPT: &str = "> ";
const DEV_CONSOLE_INPUT_GLYPH_WIDTH_FACTOR: f32 = 0.58;
const DEV_CONSOLE_CARET_WIDTH: f32 = 2.0;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DevConsoleInputLineView {
    visible_text: String,
    cursor_visible_chars: usize,
    selection_visible_chars: Option<(usize, usize)>,
}

struct DevConsoleInputLineLayout {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    font_size: f32,
    color: ColorRgba,
}

fn input_line_nodes(
    input: &DevConsoleInputSnapshot,
    layout: DevConsoleInputLineLayout,
    theme: &DevConsoleTheme,
) -> Vec<UiOverlayNode> {
    let DevConsoleInputLineLayout {
        left,
        top,
        width,
        height,
        font_size,
        color,
    } = layout;
    let view = input_line_view(input, width, font_size);
    let char_width = input_char_width(font_size);
    let prompt_width = input_text_width(DEV_CONSOLE_INPUT_PROMPT, font_size);
    let caret_x = (left + prompt_width + char_width * view.cursor_visible_chars as f32)
        .min(left + width - DEV_CONSOLE_CARET_WIDTH)
        .max(left);

    let mut nodes = Vec::new();
    if let Some((selection_start, selection_end)) = view.selection_visible_chars {
        if selection_end > selection_start {
            nodes.push(panel_node(
                "dev-console-input-selection",
                left + prompt_width + char_width * selection_start as f32,
                top + 2.0,
                char_width * (selection_end - selection_start) as f32,
                (height - 4.0).max(1.0),
                ColorRgba::new(0.15, 0.28, 0.42, 0.65),
            ));
        }
    }

    nodes.push(input_text_node(
        "dev-console-input",
        format!("{DEV_CONSOLE_INPUT_PROMPT}{}", view.visible_text),
        left,
        top,
        width,
        height,
        font_size,
        color,
        theme.text_font(),
    ));
    nodes.push(panel_node(
        "dev-console-input-caret",
        caret_x,
        top + 2.0,
        DEV_CONSOLE_CARET_WIDTH,
        (height - 4.0).max(1.0),
        color,
    ));
    nodes
}

fn input_line_view(
    input: &DevConsoleInputSnapshot,
    width: f32,
    font_size: f32,
) -> DevConsoleInputLineView {
    let prompt_width = input_text_width(DEV_CONSOLE_INPUT_PROMPT, font_size);
    let char_width = input_char_width(font_size);
    let usable_width = (width - prompt_width - DEV_CONSOLE_CARET_WIDTH).max(char_width);
    let max_visible_chars = (usable_width / char_width).floor().max(1.0) as usize;

    let text = input.text.as_str();
    let total_chars = text.chars().count();
    let cursor_chars = byte_to_char_index(text, input.cursor);

    let visible_start_chars = if total_chars <= max_visible_chars {
        0
    } else if cursor_chars >= max_visible_chars {
        cursor_chars.saturating_sub(max_visible_chars.saturating_sub(1))
    } else {
        0
    }
    .min(total_chars);

    let visible_end_chars = (visible_start_chars + max_visible_chars).min(total_chars);
    let visible_text = text
        .chars()
        .skip(visible_start_chars)
        .take(visible_end_chars.saturating_sub(visible_start_chars))
        .collect::<String>();
    let cursor_visible_chars = cursor_chars
        .saturating_sub(visible_start_chars)
        .min(visible_end_chars.saturating_sub(visible_start_chars));

    let selection_visible_chars = input.selection.and_then(|selection| {
        let selection_start = byte_to_char_index(text, selection.start);
        let selection_end = byte_to_char_index(text, selection.end);
        let start = selection_start
            .max(visible_start_chars)
            .min(visible_end_chars);
        let end = selection_end
            .max(visible_start_chars)
            .min(visible_end_chars);
        (end > start).then_some((
            start.saturating_sub(visible_start_chars),
            end.saturating_sub(visible_start_chars),
        ))
    });

    DevConsoleInputLineView {
        visible_text,
        cursor_visible_chars,
        selection_visible_chars,
    }
}

fn input_char_width(font_size: f32) -> f32 {
    (font_size * DEV_CONSOLE_INPUT_GLYPH_WIDTH_FACTOR).max(1.0)
}

fn input_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * input_char_width(font_size)
}

fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    let byte_index = byte_index.min(text.len());
    let byte_index = clamp_to_char_boundary_local(text, byte_index);
    text[..byte_index].chars().count()
}

fn clamp_to_char_boundary_local(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn text_node(
    id: impl Into<String>,
    content: String,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    font_size: f32,
    color: ColorRgba,
    font: Option<AssetKey>,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Text { content, font },
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            color: Some(color),
            font_size,
            fit_to_width: false,
            word_wrap: true,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn input_text_node(
    id: impl Into<String>,
    content: String,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    font_size: f32,
    color: ColorRgba,
    font: Option<AssetKey>,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Text { content, font },
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            color: Some(color),
            font_size,
            fit_to_width: false,
            word_wrap: false,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn completion_popup_height(completion: &ConsoleCompletionSnapshot, line_height: f32) -> f32 {
    let rows = completion.suggestions.len().min(8).max(1) as f32;
    rows * line_height + 8.0
}

fn completion_popup_node(
    completion: &ConsoleCompletionSnapshot,
    left: f32,
    top: f32,
    width: f32,
    line_height: f32,
    theme: &DevConsoleTheme,
) -> UiOverlayNode {
    let height = completion_popup_height(completion, line_height);
    let mut children = Vec::new();

    for (index, suggestion) in completion.suggestions.iter().enumerate() {
        let selected = index == completion.selected_index;
        children.push(UiOverlayNode {
            id: Some(format!("dev-console-completion-row-{index}")),
            kind: UiOverlayNodeKind::Panel,
            style: UiOverlayStyle {
                width: Some(width - 8.0),
                height: Some(line_height),
                background: Some(if selected {
                    ColorRgba::new(0.10, 0.16, 0.22, 1.0)
                } else {
                    ColorRgba::new(0.05, 0.06, 0.08, 1.0)
                }),
                ..UiOverlayStyle::default()
            },
            children: vec![text_node(
                format!("dev-console-completion-{index}"),
                format!("{}  {}", suggestion.label, suggestion.detail),
                8.0,
                0.0,
                width - 16.0,
                line_height,
                theme.font.output_size,
                if selected {
                    theme.colors.input_text
                } else {
                    theme.colors.header_text
                },
                theme.text_font(),
            )],
        });
    }

    UiOverlayNode {
        id: Some("dev-console-completion-popup".to_owned()),
        kind: UiOverlayNodeKind::Panel,
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top.max(0.0)),
            width: Some(width),
            height: Some(height),
            padding: 4.0,
            gap: 0.0,
            background: Some(ColorRgba::new(0.015, 0.020, 0.030, 1.0)),
            border_color: Some(theme.colors.panel_border),
            border_width: theme.layout.border_width,
            border_radius: theme.layout.border_radius,
            ..UiOverlayStyle::default()
        },
        children,
    }
}

fn scrollbar_nodes(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    total_entries: usize,
    visible_lines: usize,
    scroll_offset: usize,
    theme: &DevConsoleTheme,
) -> Vec<UiOverlayNode> {
    if total_entries <= visible_lines {
        return Vec::new();
    }

    let layout = &theme.layout;
    let visible_ratio = (visible_lines as f32 / total_entries as f32)
        .clamp(layout.scrollbar_min_visible_ratio, 1.0);
    let thumb_height = (height * visible_ratio)
        .max(layout.scrollbar_min_thumb_height)
        .min(height);
    let max_offset = total_entries.saturating_sub(visible_lines).max(1);
    let normalized = (scroll_offset.min(max_offset) as f32 / max_offset as f32).clamp(0.0, 1.0);
    let thumb_top = top + (height - thumb_height) * (1.0 - normalized);

    vec![
        panel_node(
            "dev-console-scrollbar-track",
            left,
            top,
            width,
            height,
            theme.colors.scrollbar_track,
        ),
        panel_node(
            "dev-console-scrollbar-thumb",
            left,
            thumb_top,
            width,
            thumb_height,
            theme.colors.scrollbar_thumb,
        ),
    ]
}

fn panel_node(
    id: impl Into<String>,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    background: ColorRgba,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Panel,
        style: UiOverlayStyle {
            left: Some(left),
            top: Some(top),
            width: Some(width),
            height: Some(height),
            background: Some(background),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use amigo_overlay_api::{build_ui_layout_tree, UiOverlayNodeKind, UiViewportSize};
    use amigo_scripting_api::DevConsoleState;

    use super::{build_dev_console_overlay, build_dev_console_overlay_with_theme, UiOverlayNode};
    use crate::dev_console_theme::DevConsoleTheme;

    #[test]
    fn console_overlay_uses_fullscreen_root_and_bounded_panel() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.write_line("hello");

        let document =
            build_dev_console_overlay(&console, None, Some(UiViewportSize::new(1280.0, 720.0)))
                .expect("overlay should be built");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &document);

        assert_eq!(layout.rect.width, 1280.0);
        assert_eq!(layout.rect.height, 720.0);
        let panel = layout
            .children
            .iter()
            .find(|child| child.node.id.as_deref() == Some("dev-console-panel"))
            .expect("panel should exist");
        assert!(panel.rect.x >= 16.0);
        assert!(panel.rect.x + panel.rect.width <= 1280.0);
    }

    #[test]
    fn console_overlay_uses_full_viewport_width() {
        let console = DevConsoleState::default();
        console.set_open(true);

        let document =
            build_dev_console_overlay(&console, None, Some(UiViewportSize::new(1920.0, 1080.0)))
                .expect("overlay should be built");
        let layout = build_ui_layout_tree(UiViewportSize::new(1920.0, 1080.0), &document);
        let panel = layout
            .children
            .iter()
            .find(|child| child.node.id.as_deref() == Some("dev-console-panel"))
            .expect("panel should exist");

        let theme = DevConsoleTheme::default();
        assert_eq!(panel.rect.width, 1920.0 - theme.layout.margin * 2.0);
    }

    #[test]
    fn console_overlay_uses_theme_layout_values() {
        let console = DevConsoleState::default();
        console.set_open(true);

        let mut theme = DevConsoleTheme::default();
        theme.layout.margin = 24.0;

        let document = build_dev_console_overlay_with_theme(
            &console,
            None,
            Some(UiViewportSize::new(1920.0, 1080.0)),
            &theme,
        )
        .expect("overlay should be built");
        let layout = build_ui_layout_tree(UiViewportSize::new(1920.0, 1080.0), &document);
        let panel = layout
            .children
            .iter()
            .find(|child| child.node.id.as_deref() == Some("dev-console-panel"))
            .expect("panel should exist");

        assert_eq!(panel.rect.width, 1920.0 - theme.layout.margin * 2.0);
        assert_eq!(panel.rect.x, 24.0);
    }

    #[test]
    fn console_overlay_text_wraps_on_narrow_viewports() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.write_line("a very long console output line that should wrap on narrow screens");

        let theme = DevConsoleTheme::default();
        let document =
            build_dev_console_overlay(&console, None, Some(UiViewportSize::new(420.0, 480.0)))
                .expect("overlay should be built");
        let layout = build_ui_layout_tree(UiViewportSize::new(420.0, 480.0), &document);
        let panel = layout
            .children
            .iter()
            .find(|child| child.node.id.as_deref() == Some("dev-console-panel"))
            .expect("panel should exist");

        assert_eq!(panel.rect.width, 420.0 - theme.layout.margin * 2.0);
        assert!(contains_wrapping_output_text_node(&document.root));
        assert!(contains_non_wrapping_input_text_node(&document.root));
    }

    #[test]
    fn console_overlay_renders_completion_popup_when_suggestions_exist() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.set_input("debug.f");

        let completion = crate::ConsoleCompletionSnapshot {
            input: "debug.f".to_owned(),
            cursor_index: "debug.f".len(),
            replacement_start: 0,
            replacement_end: "debug.f".len(),
            selected_index: 0,
            suggestions: vec![crate::ConsoleCompletionSuggestion {
                label: "debug.fps".to_owned(),
                insert_text: "debug.fps ".to_owned(),
                detail: "Show FPS.".to_owned(),
                kind: crate::ConsoleCompletionKind::Command,
            }],
        };

        let document = build_dev_console_overlay(
            &console,
            Some(&completion),
            Some(UiViewportSize::new(1280.0, 720.0)),
        )
        .expect("overlay should be built");

        assert!(contains_node_with_id(
            &document.root,
            "dev-console-completion-popup"
        ));
    }

    #[test]
    fn console_overlay_renders_caret_as_separate_node() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.set_input_with_cursor("abc", 1);

        let document =
            build_dev_console_overlay(&console, None, Some(UiViewportSize::new(1280.0, 720.0)))
                .expect("overlay should be built");

        assert!(contains_text(&document.root, "> abc"));
        assert!(!contains_text(&document.root, "> a|bc"));
        assert!(contains_node_with_id(
            &document.root,
            "dev-console-input-caret"
        ));
    }

    #[test]
    fn console_overlay_renders_selection_as_separate_node() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.set_input("opacity");
        console.move_input_home(false);
        console.move_input_right(true, false);
        console.move_input_right(true, false);

        let document =
            build_dev_console_overlay(&console, None, Some(UiViewportSize::new(1280.0, 720.0)))
                .expect("overlay should be built");

        assert!(contains_text(&document.root, "> opacity"));
        assert!(contains_node_with_id(
            &document.root,
            "dev-console-input-selection"
        ));
    }

    fn contains_node_with_id(node: &UiOverlayNode, id: &str) -> bool {
        node.id.as_deref() == Some(id)
            || node
                .children
                .iter()
                .any(|child| contains_node_with_id(child, id))
    }

    fn contains_text(node: &UiOverlayNode, expected: &str) -> bool {
        match &node.kind {
            UiOverlayNodeKind::Text { content, .. } if content == expected => true,
            _ => node
                .children
                .iter()
                .any(|child| contains_text(child, expected)),
        }
    }

    fn contains_wrapping_output_text_node(node: &UiOverlayNode) -> bool {
        match &node.kind {
            UiOverlayNodeKind::Text { .. }
                if node
                    .id
                    .as_deref()
                    .unwrap_or_default()
                    .starts_with("dev-console-output-") =>
            {
                node.style.word_wrap
            }
            _ => node.children.iter().any(contains_wrapping_output_text_node),
        }
    }

    fn contains_non_wrapping_input_text_node(node: &UiOverlayNode) -> bool {
        match &node.kind {
            UiOverlayNodeKind::Text { .. } if node.id.as_deref() == Some("dev-console-input") => {
                !node.style.word_wrap
            }
            _ => node
                .children
                .iter()
                .any(contains_non_wrapping_input_text_node),
        }
    }
}
