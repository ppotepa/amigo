use amigo_assets::{
    AssetCatalog, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
};
use amigo_editor_authoring::{
    AuthoringNode, AuthoringProperty, AuthoringPropertyApplyMode, AuthoringSceneGraph,
    AuthoringSceneGraphService, AuthoringTreeIcon, AuthoringTreeRow, InspectorViewMode,
    filter_property_panel_for_view, raw_yaml_projection, render_stack_projection,
    scene_objects_projection,
};
use amigo_math::ColorRgba;
use amigo_render_wgpu::{
    UiLayoutNode, UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind,
    UiOverlayStyle, UiOverlayViewport, UiOverlayViewportScaling, UiRect, UiViewportSize,
    WgpuRenderFramePacket, build_ui_layout_tree,
};
use amigo_runtime::Runtime;
use amigo_ui::UiInputViewportState;

use crate::layout::{
    EditorLayout, EditorScrollLayout, GAME_VIEWPORT_LOGICAL_H, GAME_VIEWPORT_LOGICAL_W, PAD, ROW_H,
};
use crate::properties::{
    as_bool, as_number, build_panel_with_overrides, display_number_with_hints, display_text,
    is_slider,
};
use crate::state::{
    EditorHitAction, EditorHitTarget, EditorRect, EditorRightPanelMode, EditorTreeMode,
    IngameEditorSnapshot, IngameEditorState,
};
use crate::theme::{
    editor_icon_font, format_compact_tags, format_property_tags, format_tags, icon_glyph,
    icon_label,
};

const TREE_TWISTY_LEAF: &str = " ";
const TREE_TWISTY_COLLAPSED: &str = "+";
const TREE_TWISTY_EXPANDED: &str = "-";

pub fn append_editor_overlay(runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
    let Some(state) = runtime.resolve::<IngameEditorState>() else {
        return;
    };

    if !state.is_open() {
        return;
    }

    ensure_editor_icon_font_asset(runtime);

    let viewport = runtime
        .resolve::<UiInputViewportState>()
        .and_then(|viewport| viewport.get())
        .unwrap_or_else(|| UiViewportSize::new(1280.0, 720.0));

    let graph = match runtime
        .resolve::<AuthoringSceneGraphService>()
        .map(|authoring| authoring.graph_for_current_scene(runtime))
    {
        Some(Ok(graph)) => graph,
        Some(Err(error)) => {
            state.set_status(format!("editor graph error: {error}"));
            empty_authoring_graph()
        }
        None => {
            state.set_status("editor authoring service is not registered");
            empty_authoring_graph()
        }
    };

    let snapshot = state.snapshot();
    let selected_id = snapshot
        .selected_node_id
        .clone()
        .or_else(|| graph.first_editable_node_id());
    let selected_node = selected_id.as_deref().and_then(|id| graph.find_node(id));

    let mut hit_targets = Vec::new();
    let mut stats = OverlayStats::default();
    let document = build_editor_document(
        viewport,
        &graph,
        selected_node,
        snapshot.status.as_str(),
        &state,
        &mut hit_targets,
        &mut stats,
    );
    sync_hit_targets_from_layout(viewport, &document, &mut hit_targets);

    state.set_scroll_bounds(stats.tree_scroll_max, stats.properties_scroll_max);
    state.set_hit_targets(hit_targets);
    packet.push_debug_overlay(document);
}

pub(crate) fn ensure_editor_icon_font_asset(runtime: &Runtime) {
    let Some(asset_catalog) = runtime.resolve::<AssetCatalog>() else {
        return;
    };
    let key = editor_icon_font();
    if !asset_catalog.contains(&key) {
        asset_catalog.register_manifest(AssetManifest {
            key: key.clone(),
            source: AssetSourceKind::Mod("core".to_owned()),
            tags: vec!["font".to_owned(), "editor".to_owned()],
        });
    }
    if !asset_catalog.is_loaded(&key)
        && !asset_catalog.is_prepared(&key)
        && !asset_catalog.is_failed(&key)
    {
        asset_catalog.request_load(AssetLoadRequest::new(key, AssetLoadPriority::Interactive));
    }
}

pub(crate) fn build_editor_document(
    viewport: UiViewportSize,
    graph: &AuthoringSceneGraph,
    selected: Option<&AuthoringNode>,
    status: &str,
    state: &IngameEditorState,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayDocument {
    let layout = EditorLayout::new(viewport);
    let snapshot = state.snapshot();
    let root_children = vec![
        top_bar(layout, graph),
        left_tree_panel(
            layout,
            graph,
            snapshot.selected_node_id.as_deref(),
            snapshot.tree_scroll,
            snapshot.tree_filter.as_str(),
            hit_targets,
            state,
            stats,
        ),
        center_viewport_panel(layout, &snapshot),
        right_panel(
            layout,
            graph,
            selected,
            state,
            snapshot.properties_scroll,
            hit_targets,
            stats,
        ),
        bottom_bar(layout, status, &snapshot),
    ];

    UiOverlayDocument {
        entity_name: "amigo-ingame-editor".to_owned(),
        layer: UiOverlayLayer::Debug,
        viewport: Some(UiOverlayViewport {
            width: viewport.width,
            height: viewport.height,
            scaling: UiOverlayViewportScaling::Expand,
        }),
        root: UiOverlayNode {
            id: Some("editor-root".to_owned()),
            kind: UiOverlayNodeKind::Stack,
            style: UiOverlayStyle {
                width: Some(viewport.width),
                height: Some(viewport.height),
                ..UiOverlayStyle::default()
            },
            children: root_children,
        },
    }
}

#[derive(Default)]
pub(crate) struct OverlayStats {
    pub(crate) tree_scroll_max: f32,
    pub(crate) properties_scroll_max: f32,
}

pub(crate) fn sync_hit_targets_from_layout(
    viewport: UiViewportSize,
    document: &UiOverlayDocument,
    hit_targets: &mut Vec<EditorHitTarget>,
) {
    let layout = build_ui_layout_tree(viewport, document);
    hit_targets.retain_mut(|target| {
        if let Some(layout_node) = find_layout_node_by_overlay_id(&layout, &target.id) {
            target.rect = editor_rect_from_ui_rect(layout_node.rect);
            true
        } else if matches!(target.action, EditorHitAction::ToggleTreeNode { .. }) {
            true
        } else {
            false
        }
    });
}

fn empty_authoring_graph() -> AuthoringSceneGraph {
    AuthoringSceneGraph {
        source_mod: "<none>".to_owned(),
        scene_id: "<none>".to_owned(),
        root_file: "<none>".into(),
        source_files: Vec::new(),
        nodes: Vec::new(),
    }
}

fn find_layout_node_by_overlay_id<'a>(
    node: &'a UiLayoutNode,
    id: &str,
) -> Option<&'a UiLayoutNode> {
    if node.node.id.as_deref() == Some(id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_layout_node_by_overlay_id(child, id))
}

fn editor_rect_from_ui_rect(rect: UiRect) -> EditorRect {
    EditorRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn panel(id: impl Into<String>, rect: EditorRect) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Column,
        style: UiOverlayStyle {
            left: Some(rect.x),
            top: Some(rect.y),
            width: Some(rect.width.max(0.0)),
            height: Some(rect.height.max(0.0)),
            padding: PAD,
            gap: 4.0,
            background: Some(ColorRgba::new(0.02, 0.025, 0.035, 0.86)),
            border_color: Some(ColorRgba::new(0.15, 0.75, 0.95, 0.35)),
            border_width: 1.0,
            border_radius: 6.0,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn text(id: impl Into<String>, content: impl Into<String>, font_size: f32) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Text {
            content: content.into(),
            font: None,
        },
        style: UiOverlayStyle {
            height: Some(ROW_H),
            color: Some(ColorRgba::new(0.82, 0.92, 1.0, 1.0)),
            font_size,
            fit_to_width: false,
            word_wrap: true,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn icon_text_node(id: impl Into<String>, icon: AuthoringTreeIcon, font_size: f32) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Text {
            content: icon_glyph(icon).to_owned(),
            font: Some(editor_icon_font()),
        },
        style: UiOverlayStyle {
            width: Some(18.0),
            height: Some(ROW_H),
            color: Some(ColorRgba::new(0.94, 0.78, 0.36, 1.0)),
            font_size,
            fit_to_width: false,
            word_wrap: false,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn slider_node(id: impl Into<String>, value: f32, min: f32, max: f32, step: f32) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Slider {
            value,
            min,
            max,
            step,
        },
        style: UiOverlayStyle {
            height: Some(ROW_H),
            width: Some(220.0),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn toggle_node(
    id: impl Into<String>,
    checked: bool,
    text_value: impl Into<String>,
) -> UiOverlayNode {
    UiOverlayNode {
        id: Some(id.into()),
        kind: UiOverlayNodeKind::Toggle {
            checked,
            text: text_value.into(),
            font: None,
        },
        style: UiOverlayStyle {
            height: Some(ROW_H),
            width: Some(220.0),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    }
}

fn left_tree_panel(
    layout: EditorLayout,
    graph: &AuthoringSceneGraph,
    selected_node_id: Option<&str>,
    tree_scroll: f32,
    tree_filter: &str,
    hit_targets: &mut Vec<EditorHitTarget>,
    state: &IngameEditorState,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    let mut node = panel("editor-left-tree", layout.left_panel.rect);
    let snapshot = state.snapshot();
    let title = match snapshot.tree_mode {
        EditorTreeMode::Clean => "SCENE OBJECTS",
        EditorTreeMode::RawYaml => "RAW YAML DEBUG",
    };
    node.children.push(text("editor-tree-title", title, 14.0));
    node.children.push(text(
        "editor-tree-mode",
        match snapshot.tree_mode {
            EditorTreeMode::Clean => "[Clean Tree]  Raw YAML",
            EditorTreeMode::RawYaml => " Clean Tree  [Raw YAML]",
        },
        11.0,
    ));
    hit_targets.push(EditorHitTarget {
        id: "editor-tree-mode-clean".to_owned(),
        rect: EditorRect {
            x: layout.left_panel.content_rect.x,
            y: layout.left_panel.content_rect.y + ROW_H,
            width: layout.left_panel.content_rect.width * 0.5,
            height: ROW_H,
        },
        action: EditorHitAction::Command {
            command: "editor.tree.clean".to_owned(),
        },
    });
    hit_targets.push(EditorHitTarget {
        id: "editor-tree-mode-raw".to_owned(),
        rect: EditorRect {
            x: layout.left_panel.content_rect.x + layout.left_panel.content_rect.width * 0.5,
            y: layout.left_panel.content_rect.y + ROW_H,
            width: layout.left_panel.content_rect.width * 0.5,
            height: ROW_H,
        },
        action: EditorHitAction::Command {
            command: "editor.tree.raw".to_owned(),
        },
    });
    let filter_label = if tree_filter.trim().is_empty() {
        "Search... use editor.tree.filter <text>".to_owned()
    } else {
        format!("Filter: {}", tree_filter.trim())
    };
    node.children
        .push(text("editor-tree-search", filter_label, 12.0));

    let mut scroll = layout.tree_scroll_layout(tree_scroll);
    let mut tree_rows = 0usize;
    if graph.nodes.is_empty() {
        node.children.push(text(
            "editor-tree-empty",
            format!(
                "No authoring nodes. mod={} scene={} root={}",
                graph.source_mod,
                graph.scene_id,
                graph.root_file.display()
            ),
            11.0,
        ));
    }
    let projection = match snapshot.tree_mode {
        EditorTreeMode::Clean => scene_objects_projection(graph),
        EditorTreeMode::RawYaml => raw_yaml_projection(graph),
    };
    push_projected_tree_rows(
        &mut node.children,
        &projection.rows,
        &mut scroll,
        layout,
        selected_node_id,
        tree_filter,
        hit_targets,
        state,
        &mut tree_rows,
    );
    let visible_rows =
        ((layout.left_panel.content_rect.height - ROW_H * 3.0) / (ROW_H + 4.0)).max(0.0);
    stats.tree_scroll_max = ((tree_rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);

    node
}

fn push_projected_tree_rows(
    children: &mut Vec<UiOverlayNode>,
    rows: &[AuthoringTreeRow],
    scroll: &mut EditorScrollLayout,
    layout: EditorLayout,
    selected_node_id: Option<&str>,
    tree_filter: &str,
    hit_targets: &mut Vec<EditorHitTarget>,
    state: &IngameEditorState,
    tree_rows: &mut usize,
) {
    let filter = tree_filter.trim();
    let mut collapsed_depths: Vec<usize> = Vec::new();
    for row in rows {
        while collapsed_depths
            .last()
            .is_some_and(|depth| *depth >= row.depth)
        {
            collapsed_depths.pop();
        }
        let hidden_by_collapse = collapsed_depths.iter().any(|depth| *depth < row.depth);
        if hidden_by_collapse {
            continue;
        }
        if !filter.is_empty() && !projected_row_matches_filter(row, filter) {
            continue;
        }

        *tree_rows += 1;
        let row_rect = layout.tree_row_rect(row.depth, scroll.render_y);
        if scroll.is_visible() {
            let id = format!("select:{}", row.node_id);
            let mut row_node = UiOverlayNode {
                id: Some(id.clone()),
                kind: UiOverlayNodeKind::Row,
                style: UiOverlayStyle {
                    left: Some(row.depth as f32 * 14.0),
                    width: Some(row_rect.width),
                    height: Some(ROW_H),
                    gap: 5.0,
                    ..UiOverlayStyle::default()
                },
                children: vec![
                    icon_text_node(format!("icon:{}", row.node_id), row.icon, 11.0),
                    text(
                        format!("label:{}", row.node_id),
                        projected_tree_row_label(row, state),
                        11.0,
                    ),
                ],
            };
            if selected_node_id == Some(row.node_id.as_str()) {
                row_node.style.background = Some(ColorRgba::new(0.08, 0.20, 0.28, 0.92));
                row_node.style.border_color = Some(ColorRgba::new(0.25, 0.85, 1.0, 0.85));
                row_node.style.border_width = 1.0;
            }
            children.push(row_node);

            if row.selectable {
                hit_targets.push(EditorHitTarget {
                    id,
                    rect: row_rect,
                    action: EditorHitAction::SelectNode {
                        node_id: row.node_id.clone(),
                        source_path: Some(row.source_path.clone()),
                        yaml_pointer: Some(row.yaml_pointer.clone()),
                    },
                });
            }

            if row.has_children {
                hit_targets.push(EditorHitTarget {
                    id: format!("toggle:{}", row.node_id),
                    rect: EditorRect {
                        x: row_rect.x,
                        y: row_rect.y,
                        width: 14.0,
                        height: row_rect.height,
                    },
                    action: EditorHitAction::ToggleTreeNode {
                        node_id: row.node_id.clone(),
                    },
                });
            }

            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }

        if filter.is_empty() && row.has_children && state.is_node_collapsed(&row.node_id) {
            collapsed_depths.push(row.depth);
        }
    }
}

fn projected_tree_row_label(row: &AuthoringTreeRow, state: &IngameEditorState) -> String {
    let twisty = if !row.has_children {
        TREE_TWISTY_LEAF
    } else if state.is_node_collapsed(&row.node_id) {
        TREE_TWISTY_COLLAPSED
    } else {
        TREE_TWISTY_EXPANDED
    };
    let left = format!("{twisty} {:<10} {}", icon_label(row.icon), row.label);
    let tags = format_tags(&row.tags);
    format!("{left:<46}{tags}")
}

fn projected_row_matches_filter(row: &AuthoringTreeRow, filter: &str) -> bool {
    let filter = filter.to_ascii_lowercase();
    row.node_id.to_ascii_lowercase().contains(&filter)
        || row.label.to_ascii_lowercase().contains(&filter)
        || row.yaml_pointer.to_ascii_lowercase().contains(&filter)
        || row
            .tags
            .iter()
            .any(|tag| tag.label.to_ascii_lowercase().contains(&filter))
}

fn right_panel(
    layout: EditorLayout,
    graph: &AuthoringSceneGraph,
    selected: Option<&AuthoringNode>,
    state: &IngameEditorState,
    properties_scroll: f32,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    match state.snapshot().right_panel_mode {
        EditorRightPanelMode::Inspector | EditorRightPanelMode::RenderStack => right_split_panel(
            layout,
            graph,
            selected,
            state,
            properties_scroll,
            hit_targets,
            stats,
        ),
        EditorRightPanelMode::RawDebug => right_raw_debug_panel(
            layout,
            graph,
            selected,
            state,
            properties_scroll,
            hit_targets,
            stats,
        ),
    }
}

fn right_split_panel(
    layout: EditorLayout,
    graph: &AuthoringSceneGraph,
    selected: Option<&AuthoringNode>,
    state: &IngameEditorState,
    properties_scroll: f32,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    let rect = layout.right_panel.rect;
    let gap = 6.0;
    let top_h = (rect.height * 0.58).max(180.0).min(rect.height - 120.0);
    let bottom_h = (rect.height - top_h - gap).max(0.0);
    let mut top_layout = layout;
    top_layout.right_panel = split_panel_layout(rect.x, rect.y, rect.width, top_h);
    let mut bottom_layout = layout;
    bottom_layout.right_panel =
        split_panel_layout(rect.x, rect.y + top_h + gap, rect.width, bottom_h);

    let mut top_stats = OverlayStats::default();
    let mut bottom_stats = OverlayStats::default();
    let children = vec![
        right_properties_panel(
            top_layout,
            graph,
            selected,
            state,
            properties_scroll,
            hit_targets,
            &mut top_stats,
        ),
        right_render_stack_panel(
            bottom_layout,
            graph,
            state,
            0.0,
            hit_targets,
            &mut bottom_stats,
        ),
    ];
    stats.properties_scroll_max = top_stats
        .properties_scroll_max
        .max(bottom_stats.properties_scroll_max);

    UiOverlayNode {
        id: Some("editor-right-split".to_owned()),
        kind: UiOverlayNodeKind::Stack,
        style: UiOverlayStyle {
            left: Some(rect.x),
            top: Some(rect.y),
            width: Some(rect.width),
            height: Some(rect.height),
            ..UiOverlayStyle::default()
        },
        children,
    }
}

fn split_panel_layout(x: f32, y: f32, width: f32, height: f32) -> crate::layout::EditorPanelLayout {
    let rect = EditorRect {
        x,
        y,
        width: width.max(0.0),
        height: height.max(0.0),
    };
    crate::layout::EditorPanelLayout {
        rect,
        content_rect: EditorRect {
            x: rect.x + PAD,
            y: rect.y + PAD,
            width: (rect.width - PAD * 2.0).max(0.0),
            height: (rect.height - PAD * 2.0).max(0.0),
        },
    }
}

fn push_right_tabs(children: &mut Vec<UiOverlayNode>, active: EditorRightPanelMode) {
    let label = match active {
        EditorRightPanelMode::Inspector | EditorRightPanelMode::RenderStack => {
            "[Inspector / Properties]        Raw Debug"
        }
        EditorRightPanelMode::RawDebug => " Inspector / Properties        [Raw Debug]",
    };
    children.push(text("editor-right-tabs", label, 11.0));
}

fn push_right_tab_hit_targets(layout: EditorLayout, hit_targets: &mut Vec<EditorHitTarget>) {
    let y = layout.right_panel.content_rect.y;
    let x = layout.right_panel.content_rect.x;
    let h = ROW_H;
    let w = layout.right_panel.content_rect.width / 2.0;
    for (i, (id, command)) in [
        ("tab:inspector", "editor.panel.inspector"),
        ("tab:raw-debug", "editor.panel.raw_debug"),
    ]
    .into_iter()
    .enumerate()
    {
        hit_targets.push(EditorHitTarget {
            id: id.to_owned(),
            rect: EditorRect {
                x: x + w * i as f32,
                y,
                width: w,
                height: h,
            },
            action: EditorHitAction::Command {
                command: command.to_owned(),
            },
        });
    }
}

fn right_properties_panel(
    layout: EditorLayout,
    graph: &AuthoringSceneGraph,
    selected: Option<&AuthoringNode>,
    state: &IngameEditorState,
    properties_scroll: f32,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    let mut node = panel("editor-properties", layout.right_panel.rect);
    push_right_tabs(&mut node.children, state.snapshot().right_panel_mode);
    push_right_tab_hit_targets(layout, hit_targets);

    let Some(selected) = selected else {
        node.children
            .push(text("editor-properties-empty", "Select scene object", 13.0));
        return node;
    };

    let panel = filter_property_panel_for_view(
        build_panel_with_overrides(selected, |property_id| state.override_value(property_id)),
        InspectorViewMode::Primary,
    );
    node.children
        .push(text("editor-properties-title", panel.title.clone(), 14.0));
    let breadcrumb = graph.breadcrumb_for_node(&selected.id).join(" > ");
    if !breadcrumb.is_empty() {
        node.children.push(text(
            "editor-properties-breadcrumb",
            format!("path: {breadcrumb}"),
            10.0,
        ));
    }
    let snapshot = state.snapshot();
    if !is_tree_node_visible(
        graph,
        &selected.id,
        snapshot.tree_filter.as_str(),
        &snapshot.collapsed_node_ids,
    ) {
        node.children.push(text(
            "editor-properties-hidden-selection",
            "selected node is hidden by tree collapse/filter",
            10.0,
        ));
    }

    if panel.groups.is_empty() {
        node.children.push(text(
            "editor-properties-no-controls",
            "No descriptor-backed controls for this selection",
            11.0,
        ));
        node.children.push(text(
            "editor-properties-no-controls-hint",
            "Select an entity/component or a Draw Layer in Render Stack",
            10.0,
        ));
        return node;
    }

    let mut scroll = layout.properties_scroll_layout(properties_scroll);
    let mut property_rows = 0usize;
    for group in panel.groups {
        property_rows += 1;
        if scroll.is_visible() {
            node.children
                .push(text(format!("group:{}", group.title), group.title, 12.0));
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }

        for row in group.properties {
            property_rows += property_visual_row_count(&row.editor);
            push_property_row(&mut node.children, layout, &mut scroll, row, hit_targets);
        }
    }
    let visible_rows =
        ((layout.right_panel.content_rect.height - ROW_H * 3.0) / (ROW_H + 4.0)).max(0.0);
    stats.properties_scroll_max = ((property_rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);

    node
}

fn right_raw_debug_panel(
    layout: EditorLayout,
    _graph: &AuthoringSceneGraph,
    selected: Option<&AuthoringNode>,
    state: &IngameEditorState,
    properties_scroll: f32,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    let mut node = panel("editor-raw-debug", layout.right_panel.rect);
    push_right_tabs(&mut node.children, EditorRightPanelMode::RawDebug);
    push_right_tab_hit_targets(layout, hit_targets);
    node.children
        .push(text("raw-debug-title", "RAW DEBUG", 13.0));

    let Some(selected) = selected else {
        node.children
            .push(text("editor-raw-debug-empty", "Select scene object", 13.0));
        return node;
    };

    node.children.push(text(
        "raw-debug-source",
        format!(
            "{} | {} | {:?} | editable={}",
            selected.source_file.display(),
            selected.yaml_pointer,
            selected.kind,
            selected.editable
        ),
        10.0,
    ));

    let panel = filter_property_panel_for_view(
        build_panel_with_overrides(selected, |property_id| state.override_value(property_id)),
        InspectorViewMode::RawDebug,
    );
    let mut scroll = layout.properties_scroll_layout(properties_scroll);
    let mut property_rows = 0usize;
    for group in panel.groups {
        property_rows += 1;
        if scroll.is_visible() {
            node.children.push(text(
                format!("raw-group:{}", group.title),
                group.title,
                12.0,
            ));
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }
        for row in group.properties {
            property_rows += property_visual_row_count(&row.editor);
            push_property_row(&mut node.children, layout, &mut scroll, row, hit_targets);
        }
    }
    let visible_rows =
        ((layout.right_panel.content_rect.height - ROW_H * 3.0) / (ROW_H + 4.0)).max(0.0);
    stats.properties_scroll_max = ((property_rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);
    node
}

fn right_render_stack_panel(
    layout: EditorLayout,
    graph: &AuthoringSceneGraph,
    _state: &IngameEditorState,
    properties_scroll: f32,
    hit_targets: &mut Vec<EditorHitTarget>,
    stats: &mut OverlayStats,
) -> UiOverlayNode {
    let mut node = panel("editor-render-stack", layout.right_panel.rect);
    node.children.push(text(
        "render-stack-title",
        "RENDER STACK / DRAW LAYERS",
        13.0,
    ));

    let layers = render_stack_projection(graph).layers;
    let mut scroll = layout.properties_scroll_layout(properties_scroll);
    let mut rows = 0usize;

    for layer in layers {
        rows += 1;
        if scroll.is_visible() {
            let row_rect = layout.property_row_rect(scroll.render_y);
            let label = format!(
                "Layer {:<24} [order {:.0}][op {:.0}%]{}",
                layer.id,
                layer.order,
                layer.opacity * 100.0,
                if layer.visible {
                    "[Visible]"
                } else {
                    "[Hidden]"
                }
            );
            node.children
                .push(text(format!("render-layer:{}", layer.node_id), label, 11.0));
            hit_targets.push(EditorHitTarget {
                id: format!("render-layer-hit:{}", layer.node_id),
                rect: row_rect,
                action: EditorHitAction::SelectNode {
                    node_id: layer.node_id.clone(),
                    source_path: Some(layer.source_path.clone()),
                    yaml_pointer: Some(layer.yaml_pointer.clone()),
                },
            });
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }

        for entity in layer.entities {
            rows += 1;
            if scroll.is_visible() {
                let row_rect = layout.property_row_rect(scroll.render_y);
                let tags = format_compact_tags(&entity.tags);
                node.children.push(text(
                    format!("render-entity:{}", entity.node_id),
                    format!(
                        "  {:<10} {:<24} {}",
                        icon_label(entity.icon),
                        entity.label,
                        tags
                    ),
                    10.5,
                ));
                hit_targets.push(EditorHitTarget {
                    id: format!("render-entity-hit:{}", entity.node_id),
                    rect: row_rect,
                    action: EditorHitAction::SelectNode {
                        node_id: entity.node_id.clone(),
                        source_path: Some(entity.source_path.clone()),
                        yaml_pointer: Some(entity.yaml_pointer.clone()),
                    },
                });
                scroll.advance_rendered();
            } else {
                scroll.advance_virtual();
            }
        }
    }

    let visible_rows =
        ((layout.right_panel.content_rect.height - ROW_H * 3.0) / (ROW_H + 4.0)).max(0.0);
    stats.properties_scroll_max = ((rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);
    node
}

fn property_visual_row_count(editor: &amigo_editor_authoring::AuthoringPropertyEditor) -> usize {
    if is_slider(editor).is_some() { 2 } else { 1 }
}

fn property_label(row: &AuthoringProperty) -> String {
    let tags = if row.display.tags.is_empty() {
        String::new()
    } else {
        format!(" {}", format_property_tags(&row.display.tags))
    };
    format!("{}{}", row.label, tags)
}

fn property_value_suffix(row: &AuthoringProperty) -> String {
    if let Some(value) = as_number(&row.value) {
        return format!(" = {}", display_number_with_hints(value, &row.hints));
    }
    String::new()
}

fn push_property_row(
    children: &mut Vec<UiOverlayNode>,
    layout: EditorLayout,
    scroll: &mut EditorScrollLayout,
    row: AuthoringProperty,
    hit_targets: &mut Vec<EditorHitTarget>,
) {
    let row_id = format!("property:{}", row.id);
    let interactive = property_has_editable_hit_target(&row);
    if let (Some((min, max, step)), Some(value)) = (is_slider(&row.editor), as_number(&row.value)) {
        if scroll.is_visible() {
            children.push(text(
                format!("label:{}", row.id),
                format!("{}{}", property_label(&row), property_value_suffix(&row)),
                11.0,
            ));
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }
        if scroll.is_visible() {
            let row_rect = layout.property_row_rect(scroll.render_y);
            children.push(slider_node(row_id.clone(), value, min, max, step));
            if interactive {
                hit_targets.push(EditorHitTarget {
                    id: row_id,
                    rect: row_rect,
                    action: EditorHitAction::Slider {
                        property_id: row.id,
                        target: row.binding,
                        min,
                        max,
                        current: value,
                    },
                });
            }
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }
        return;
    }
    if scroll.is_visible() {
        let row_rect = layout.property_row_rect(scroll.render_y);
        match &row.editor {
            amigo_editor_authoring::AuthoringPropertyEditor::Toggle => {
                if let Some(value) = as_bool(&row.value) {
                    children.push(toggle_node(row_id.clone(), value, property_label(&row)));
                    if interactive {
                        hit_targets.push(EditorHitTarget {
                            id: row_id,
                            rect: row_rect,
                            action: EditorHitAction::Toggle {
                                property_id: row.id,
                                target: row.binding,
                                current: value,
                            },
                        });
                    }
                } else {
                    children.push(text(
                        row_id.clone(),
                        display_text(&property_label(&row), &row.value),
                        11.0,
                    ));
                }
            }
            amigo_editor_authoring::AuthoringPropertyEditor::Number => {
                let label = display_text(&property_label(&row), &row.value);
                children.push(text(row_id.clone(), label, 11.0));
                if interactive && let Some(value) = as_number(&row.value) {
                    let next = next_number_value(value, &row);
                    hit_targets.push(EditorHitTarget {
                        id: row_id,
                        rect: row_rect,
                        action: EditorHitAction::NumberCommit {
                            property_id: row.id,
                            target: row.binding,
                            value: next,
                        },
                    });
                }
            }
            amigo_editor_authoring::AuthoringPropertyEditor::Enum { options } => {
                let label = display_text(&property_label(&row), &row.value);
                children.push(text(row_id.clone(), label, 11.0));
                if interactive && let Some(next) = next_enum_value(&row.value, options) {
                    hit_targets.push(EditorHitTarget {
                        id: row_id,
                        rect: row_rect,
                        action: EditorHitAction::EnumSelect {
                            property_id: row.id,
                            target: row.binding,
                            value: next,
                        },
                    });
                }
            }
            amigo_editor_authoring::AuthoringPropertyEditor::Text => {
                children.push(text(
                    row_id.clone(),
                    display_text(&property_label(&row), &row.value),
                    11.0,
                ));
            }
            amigo_editor_authoring::AuthoringPropertyEditor::AssetPicker { .. } => {
                children.push(text(
                    row_id.clone(),
                    display_text(&property_label(&row), &row.value),
                    11.0,
                ));
            }
            _ => {
                children.push(text(
                    row_id.clone(),
                    display_text(&property_label(&row), &row.value),
                    11.0,
                ));
            }
        }
        scroll.advance_rendered();
    } else {
        scroll.advance_virtual();
    }
}

pub(crate) fn property_has_editable_hit_target(row: &AuthoringProperty) -> bool {
    matches!(
        row.display.apply_mode,
        AuthoringPropertyApplyMode::Live | AuthoringPropertyApplyMode::Mock
    ) && row.binding.is_some()
        && !row.read_only
        && matches!(
            row.editor,
            amigo_editor_authoring::AuthoringPropertyEditor::Slider { .. }
                | amigo_editor_authoring::AuthoringPropertyEditor::Toggle
                | amigo_editor_authoring::AuthoringPropertyEditor::Number
                | amigo_editor_authoring::AuthoringPropertyEditor::Enum { .. }
        )
}

fn text_value(value: &amigo_editor_authoring::AuthoringPropertyValue) -> Option<String> {
    match value {
        amigo_editor_authoring::AuthoringPropertyValue::Text(value)
        | amigo_editor_authoring::AuthoringPropertyValue::AssetRef(value)
        | amigo_editor_authoring::AuthoringPropertyValue::Enum(value)
        | amigo_editor_authoring::AuthoringPropertyValue::Color(value) => Some(value.clone()),
        _ => None,
    }
}

fn next_enum_value(
    value: &amigo_editor_authoring::AuthoringPropertyValue,
    options: &[String],
) -> Option<String> {
    let current = text_value(value)?;
    if options.is_empty() {
        return None;
    }
    let index = options.iter().position(|option| option == &current);
    Some(options[(index.map(|i| i + 1).unwrap_or(0)) % options.len()].clone())
}

fn next_number_value(current: f32, row: &AuthoringProperty) -> f32 {
    let Some(number) = &row.hints.number else {
        return current + 1.0;
    };
    let step = number.step.unwrap_or(1.0);
    let mut next = current + step;
    if let Some(max) = number.max {
        if next > max {
            next = number.min.unwrap_or(max);
        }
    }
    if let Some(min) = number.min {
        next = next.max(min);
    }
    next
}

#[cfg(test)]
pub(crate) fn collect_render_stack(
    graph: &AuthoringSceneGraph,
) -> Vec<amigo_editor_authoring::RenderStackLayerRow> {
    render_stack_projection(graph).layers
}

fn node_or_descendant_matches_filter(node: &AuthoringNode, filter: &str) -> bool {
    node_matches_filter(node, filter)
        || node
            .children
            .iter()
            .any(|child| node_or_descendant_matches_filter(child, filter))
}

pub(crate) fn is_tree_node_visible(
    graph: &AuthoringSceneGraph,
    node_id: &str,
    tree_filter: &str,
    collapsed_node_ids: &std::collections::BTreeSet<String>,
) -> bool {
    graph.nodes.iter().any(|node| {
        is_tree_node_visible_inner(node, node_id, tree_filter.trim(), collapsed_node_ids)
    })
}

fn is_tree_node_visible_inner(
    node: &AuthoringNode,
    node_id: &str,
    tree_filter: &str,
    collapsed_node_ids: &std::collections::BTreeSet<String>,
) -> bool {
    if !tree_filter.is_empty() && !node_or_descendant_matches_filter(node, tree_filter) {
        return false;
    }
    if node.id == node_id {
        return true;
    }
    let force_expanded_by_filter = !tree_filter.is_empty();
    if !force_expanded_by_filter && collapsed_node_ids.contains(&node.id) {
        return false;
    }
    node.children
        .iter()
        .any(|child| is_tree_node_visible_inner(child, node_id, tree_filter, collapsed_node_ids))
}

fn node_matches_filter(node: &AuthoringNode, filter: &str) -> bool {
    let filter = filter.to_ascii_lowercase();
    let kind = format!("{:?}", node.kind).to_ascii_lowercase();
    let origin = format!("{:?}", node.origin).to_ascii_lowercase();
    node.id.to_ascii_lowercase().contains(&filter)
        || node.label.to_ascii_lowercase().contains(&filter)
        || node.yaml_pointer.to_ascii_lowercase().contains(&filter)
        || node.value_preview.to_ascii_lowercase().contains(&filter)
        || kind.contains(&filter)
        || origin.contains(&filter)
        || node
            .semantic
            .owner_entity_name
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
        || node
            .semantic
            .component_type
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
        || node
            .semantic
            .render_layer_id
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
        || node
            .semantic
            .post_fx_id
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
        || node
            .semantic
            .light_group_id
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
        || node
            .semantic
            .light_route_receiver_layer
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&filter)
}

fn top_bar(layout: EditorLayout, graph: &AuthoringSceneGraph) -> UiOverlayNode {
    let mut node = panel("editor-top-bar", layout.top_bar.rect);
    node.children.push(text(
        "editor-title",
        format!(
            "AMIGO EDITOR MOCKUP | mod {} | scene {} | F3 / Ctrl+E toggle | ` console",
            graph.source_mod, graph.scene_id
        ),
        13.0,
    ));
    node
}

fn center_viewport_panel(layout: EditorLayout, snapshot: &IngameEditorSnapshot) -> UiOverlayNode {
    let mut node = UiOverlayNode {
        id: Some("editor-center-viewport".to_owned()),
        kind: UiOverlayNodeKind::Stack,
        style: UiOverlayStyle {
            left: Some(layout.center_panel.rect.x),
            top: Some(layout.center_panel.rect.y),
            width: Some(layout.center_panel.rect.width),
            height: Some(layout.center_panel.rect.height),
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    };

    let game_layout = layout.game_viewport_layout();
    let game_rect = game_layout.rect;
    node.children.push(UiOverlayNode {
        id: Some("editor-game-viewport-wrapper".to_owned()),
        kind: UiOverlayNodeKind::Stack,
        style: UiOverlayStyle {
            left: Some(game_rect.x),
            top: Some(game_rect.y),
            width: Some(game_rect.width),
            height: Some(game_rect.height),
            background: Some(ColorRgba::new(0.0, 0.0, 0.0, 0.0)),
            border_color: Some(ColorRgba::new(0.25, 0.85, 1.0, 0.28)),
            border_width: 1.0,
            ..UiOverlayStyle::default()
        },
        children: Vec::new(),
    });

    if let Some(selection) = &snapshot.viewport_selection {
        if let Some(bounds) = selection.logical_bounds {
            let screen_bounds = game_layout.logical_rect_to_screen_with_view(
                bounds,
                snapshot.viewport_pan_x,
                snapshot.viewport_pan_y,
                snapshot.viewport_zoom,
            );
            node.children.push(UiOverlayNode {
                id: Some("editor-selected-bounds".to_owned()),
                kind: UiOverlayNodeKind::Stack,
                style: UiOverlayStyle {
                    left: Some(screen_bounds.x),
                    top: Some(screen_bounds.y),
                    width: Some(screen_bounds.width),
                    height: Some(screen_bounds.height),
                    background: Some(ColorRgba::new(0.0, 0.0, 0.0, 0.0)),
                    border_color: Some(ColorRgba::new(1.0, 0.86, 0.18, 0.92)),
                    border_width: 2.0,
                    ..UiOverlayStyle::default()
                },
                children: Vec::new(),
            });
        }
    }

    node
}

fn bottom_bar(
    layout: EditorLayout,
    status: &str,
    snapshot: &IngameEditorSnapshot,
) -> UiOverlayNode {
    let mut node = panel("editor-bottom-bar", layout.bottom_bar.rect);
    let game_rect = layout.game_viewport_rect();
    node.children.push(text(
        "editor-status",
        format!(
            "status: {status} | game viewport {:.0}x{:.0} -> {:.0}x{:.0} | pan {:.1},{:.1} zoom {:.2} | MMB drag: pan viewport",
            GAME_VIEWPORT_LOGICAL_W,
            GAME_VIEWPORT_LOGICAL_H,
            game_rect.width,
            game_rect.height,
            snapshot.viewport_pan_x,
            snapshot.viewport_pan_y,
            snapshot.viewport_zoom
        ),
        11.0,
    ));
    node
}
