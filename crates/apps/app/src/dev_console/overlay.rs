use amigo_assets::AssetKey;
use amigo_math::ColorRgba;
use amigo_render_wgpu::{
    UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle,
    UiOverlayViewport, UiOverlayViewportScaling, UiViewportSize,
};
use amigo_scripting_api::{DevConsoleOutputLine, DevConsoleState};

use super::theme::DevConsoleTheme;

pub(crate) fn build_dev_console_overlay(
    console: &DevConsoleState,
    viewport: Option<UiViewportSize>,
) -> Option<UiOverlayDocument> {
    build_dev_console_overlay_with_theme(console, viewport, &DevConsoleTheme::default())
}

pub(crate) fn build_dev_console_overlay_with_theme(
    console: &DevConsoleState,
    viewport: Option<UiViewportSize>,
    theme: &DevConsoleTheme,
) -> Option<UiOverlayDocument> {
    if !console.is_open() {
        return None;
    }

    let viewport = viewport.unwrap_or_else(|| {
        UiViewportSize::new(
            theme.viewport.fallback_width,
            theme.viewport.fallback_height,
        )
    });
    let layout = &theme.layout;
    let panel_width = (viewport.width - layout.margin * 2.0)
        .clamp(layout.min_panel_width, layout.max_panel_width);
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
    let input = console.input();

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
            children: vec![
                backdrop_node(viewport, theme),
                console_panel_node(
                    panel_left,
                    panel_top,
                    panel_width,
                    panel_height,
                    content_width,
                    output,
                    total_entries,
                    visible_lines,
                    scroll_offset,
                    input,
                    theme,
                ),
            ],
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

#[allow(clippy::too_many_arguments)]
fn console_panel_node(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    content_width: f32,
    output: Vec<DevConsoleOutputLine>,
    total_entries: usize,
    visible_lines: usize,
    scroll_offset: usize,
    input: String,
    theme: &DevConsoleTheme,
) -> UiOverlayNode {
    let layout = &theme.layout;
    let mut children = Vec::new();
    children.push(text_node(
        "dev-console-header",
        format!(
            "AMIGO DEV CONSOLE  lines={} scroll={}  mouse wheel scrolls output",
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

    children.push(text_node(
        "dev-console-input",
        format!("> {input}_"),
        0.0,
        height - layout.panel_padding * 2.0 - layout.input_height,
        content_width,
        layout.input_height,
        theme.font.input_size,
        theme.colors.input_text,
        theme.text_font(),
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
            fit_to_width: true,
            word_wrap: false,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
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
    use amigo_render_wgpu::{UiViewportSize, build_ui_layout_tree};
    use amigo_scripting_api::DevConsoleState;

    use super::{build_dev_console_overlay, build_dev_console_overlay_with_theme};
    use crate::dev_console::theme::DevConsoleTheme;

    #[test]
    fn console_overlay_uses_fullscreen_root_and_bounded_panel() {
        let console = DevConsoleState::default();
        console.set_open(true);
        console.write_line("hello");

        let document =
            build_dev_console_overlay(&console, Some(UiViewportSize::new(1280.0, 720.0)))
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
    fn console_overlay_caps_fullscreen_panel_width() {
        let console = DevConsoleState::default();
        console.set_open(true);

        let document =
            build_dev_console_overlay(&console, Some(UiViewportSize::new(1920.0, 1080.0)))
                .expect("overlay should be built");
        let layout = build_ui_layout_tree(UiViewportSize::new(1920.0, 1080.0), &document);
        let panel = layout
            .children
            .iter()
            .find(|child| child.node.id.as_deref() == Some("dev-console-panel"))
            .expect("panel should exist");

        let theme = DevConsoleTheme::default();
        assert_eq!(panel.rect.width, theme.layout.max_panel_width);
    }

    #[test]
    fn console_overlay_uses_theme_layout_values() {
        let console = DevConsoleState::default();
        console.set_open(true);

        let mut theme = DevConsoleTheme::default();
        theme.layout.max_panel_width = 900.0;
        theme.layout.margin = 24.0;

        let document = build_dev_console_overlay_with_theme(
            &console,
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

        assert_eq!(panel.rect.width, 900.0);
        assert_eq!(panel.rect.x, 24.0);
    }
}
