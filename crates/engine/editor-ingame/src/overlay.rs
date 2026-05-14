use amigo_editor_authoring::{
    AuthoringNode, AuthoringNodeKind, AuthoringSceneGraph, AuthoringSceneGraphService,
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
use crate::properties::{as_bool, as_number, build_panel_with_overrides, display_text, is_slider};
use crate::state::{
    EditorHitAction, EditorHitTarget, EditorRect, IngameEditorSnapshot, IngameEditorState,
};

const TREE_TWISTY_LEAF: &str = " ";
const TREE_TWISTY_COLLAPSED: &str = "\u{25B8}";
const TREE_TWISTY_EXPANDED: &str = "\u{25BE}";

pub fn append_editor_overlay(runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
    let Some(state) = runtime.resolve::<IngameEditorState>() else {
        return;
    };

    if !state.is_open() {
        return;
    }

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

fn build_editor_document(
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
        right_properties_panel(
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
struct OverlayStats {
    tree_scroll_max: f32,
    properties_scroll_max: f32,
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
    node.children
        .push(text("editor-tree-title", "YAML TREE", 14.0));
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
    for root in &graph.nodes {
        push_tree_rows(
            &mut node.children,
            root,
            0,
            &mut scroll,
            layout,
            selected_node_id,
            tree_filter,
            hit_targets,
            state,
            &mut tree_rows,
        );
    }
    let visible_rows =
        ((layout.left_panel.content_rect.height - ROW_H * 2.0) / (ROW_H + 4.0)).max(0.0);
    stats.tree_scroll_max = ((tree_rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);

    node
}

fn push_tree_rows(
    children: &mut Vec<UiOverlayNode>,
    node: &AuthoringNode,
    depth: usize,
    scroll: &mut EditorScrollLayout,
    layout: EditorLayout,
    selected_node_id: Option<&str>,
    tree_filter: &str,
    hit_targets: &mut Vec<EditorHitTarget>,
    state: &IngameEditorState,
    tree_rows: &mut usize,
) {
    let filter = tree_filter.trim();
    if !filter.is_empty() && !node_or_descendant_matches_filter(node, filter) {
        return;
    }
    *tree_rows += 1;
    let row_rect = layout.tree_row_rect(depth, scroll.render_y);
    if scroll.is_visible() {
        let id = format!("select:{}", node.id);
        let mut row = text(id.clone(), tree_row_label(node, state), 11.0);
        row.style.left = Some(depth as f32 * 14.0);
        row.style.width = Some(row_rect.width);
        if selected_node_id == Some(node.id.as_str()) {
            row.style.background = Some(ColorRgba::new(0.08, 0.20, 0.28, 0.92));
            row.style.border_color = Some(ColorRgba::new(0.25, 0.85, 1.0, 0.85));
            row.style.border_width = 1.0;
        }
        children.push(row);

        hit_targets.push(EditorHitTarget {
            id,
            rect: row_rect,
            action: EditorHitAction::SelectNode {
                node_id: node.id.clone(),
                source_path: Some(node.source_file.display().to_string()),
                yaml_pointer: Some(node.yaml_pointer.clone()),
            },
        });

        if !node.children.is_empty() {
            hit_targets.push(EditorHitTarget {
                id: format!("toggle:{}", node.id),
                rect: EditorRect {
                    x: row_rect.x,
                    y: row_rect.y,
                    width: 14.0,
                    height: row_rect.height,
                },
                action: EditorHitAction::ToggleTreeNode {
                    node_id: node.id.clone(),
                },
            });
        }

        scroll.advance_rendered();
    } else {
        scroll.advance_virtual();
    }

    let force_expanded_by_filter = !filter.is_empty();
    if force_expanded_by_filter || !state.is_node_collapsed(&node.id) {
        for child in &node.children {
            push_tree_rows(
                children,
                child,
                depth + 1,
                scroll,
                layout,
                selected_node_id,
                tree_filter,
                hit_targets,
                state,
                tree_rows,
            );
        }
    }
}

pub(crate) fn tree_row_label(node: &AuthoringNode, state: &IngameEditorState) -> String {
    let twisty = if node.children.is_empty() {
        TREE_TWISTY_LEAF
    } else if state.is_node_collapsed(&node.id) {
        TREE_TWISTY_COLLAPSED
    } else {
        TREE_TWISTY_EXPANDED
    };
    format!("{twisty} {} {}", kind_marker_for_node(node), node.label)
}

fn kind_marker_for_node(node: &AuthoringNode) -> &'static str {
    match node.kind {
        AuthoringNodeKind::File => "[file]",
        AuthoringNodeKind::Use => "[use]",
        AuthoringNodeKind::RenderLayer => "[layer]",
        AuthoringNodeKind::LightGroup => "[light]",
        AuthoringNodeKind::LightRoute => "[route]",
        AuthoringNodeKind::Entity => "[entity]",
        AuthoringNodeKind::Component => "[component]",
        AuthoringNodeKind::PostFxItem => "[fx]",
        AuthoringNodeKind::PrefabRef => "[prefab]",
        AuthoringNodeKind::PrefabOverrides => "[overrides]",
        _ => "-",
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

    let Some(selected) = selected else {
        node.children
            .push(text("editor-properties-empty", "Select YAML node", 13.0));
        return node;
    };

    let panel =
        build_panel_with_overrides(selected, |property_id| state.override_value(property_id));
    node.children
        .push(text("editor-properties-title", panel.title, 14.0));
    let breadcrumb = graph.breadcrumb_for_node(&selected.id).join(" > ");
    if !breadcrumb.is_empty() {
        node.children.push(text(
            "editor-properties-breadcrumb",
            format!("path: {breadcrumb}"),
            10.0,
        ));
    }
    node.children.push(text(
        "editor-properties-source",
        format!(
            "{} | {} | {:?} | editable={}",
            selected.source_file.display(),
            selected.yaml_pointer,
            selected.kind,
            selected.editable
        ),
        10.0,
    ));
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
    let visible_rows = ((layout.right_panel.content_rect.height - ROW_H) / (ROW_H + 4.0)).max(0.0);
    stats.properties_scroll_max = ((property_rows as f32 - visible_rows).max(0.0)) * (ROW_H + 4.0);

    node
}

fn property_visual_row_count(editor: &amigo_editor_authoring::AuthoringPropertyEditor) -> usize {
    if is_slider(editor).is_some() { 2 } else { 1 }
}

fn push_property_row(
    children: &mut Vec<UiOverlayNode>,
    layout: EditorLayout,
    scroll: &mut EditorScrollLayout,
    row: amigo_editor_authoring::AuthoringProperty,
    hit_targets: &mut Vec<EditorHitTarget>,
) {
    let row_id = format!("property:{}", row.id);
    if let (Some((min, max, step)), Some(value)) = (is_slider(&row.editor), as_number(&row.value)) {
        if scroll.is_visible() {
            children.push(text(format!("label:{}", row.id), row.label.clone(), 11.0));
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }
        if scroll.is_visible() {
            let row_rect = layout.property_row_rect(scroll.render_y);
            children.push(slider_node(row_id.clone(), value, min, max, step));
            hit_targets.push(EditorHitTarget {
                id: row_id,
                rect: row_rect,
                action: EditorHitAction::Slider {
                    property_id: row.id,
                    target: row.binding,
                    min,
                    max,
                },
            });
            scroll.advance_rendered();
        } else {
            scroll.advance_virtual();
        }
        return;
    }
    if scroll.is_visible() {
        let row_rect = layout.property_row_rect(scroll.render_y);
        if let Some(value) = as_bool(&row.value) {
            children.push(toggle_node(row_id.clone(), value, row.label.clone()));
            hit_targets.push(EditorHitTarget {
                id: row_id,
                rect: row_rect,
                action: EditorHitAction::Toggle {
                    property_id: row.id,
                    target: row.binding,
                    current: value,
                },
            });
        } else {
            children.push(text(
                row_id.clone(),
                display_text(&row.label, &row.value),
                11.0,
            ));
        }
        scroll.advance_rendered();
    } else {
        scroll.advance_virtual();
    }
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
