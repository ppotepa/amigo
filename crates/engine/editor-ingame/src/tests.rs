use super::*;

#[test]
fn editor_state_toggles_open_closed() {
    let state = IngameEditorState::new(true);

    assert!(state.is_open());
    state.toggle();
    assert!(!state.is_open());
    state.toggle();
    assert!(state.is_open());
}

#[test]
fn editor_state_stores_mock_override() {
    let state = IngameEditorState::new(true);

    state.set_override(
        "render_layer.background.city.opacity",
        state::EditorPropertyValue::Number(0.5),
    );

    assert_eq!(
        state.override_value("render_layer.background.city.opacity"),
        Some(state::EditorPropertyValue::Number(0.5))
    );
}

fn test_node(
    id: &str,
    label: &str,
    kind: amigo_editor_authoring::AuthoringNodeKind,
    yaml_pointer: &str,
    yaml: &str,
    semantic: amigo_editor_authoring::AuthoringNodeSemantic,
) -> amigo_editor_authoring::AuthoringNode {
    amigo_editor_authoring::AuthoringNode {
        id: id.to_owned(),
        label: label.to_owned(),
        kind,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: yaml_pointer.to_owned(),
        editable: true,
        value: serde_yaml::from_str(yaml).expect("test yaml"),
        value_preview: "mapping".to_owned(),
        semantic,
        children: Vec::new(),
    }
}

fn test_graph(
    nodes: Vec<amigo_editor_authoring::AuthoringNode>,
) -> amigo_editor_authoring::AuthoringSceneGraph {
    amigo_editor_authoring::AuthoringSceneGraph {
        source_mod: "test".to_owned(),
        scene_id: "test".to_owned(),
        root_file: "scene.yml".into(),
        source_files: vec!["scene.yml".into()],
        nodes,
    }
}

fn find_overlay_text<'a>(node: &'a amigo_render_wgpu::UiOverlayNode, id: &str) -> Option<&'a str> {
    if node.id.as_deref() == Some(id) {
        if let amigo_render_wgpu::UiOverlayNodeKind::Text { content, .. } = &node.kind {
            return Some(content.as_str());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_overlay_text(child, id))
}

fn find_overlay_node<'a>(
    node: &'a amigo_render_wgpu::UiOverlayNode,
    id: &str,
) -> Option<&'a amigo_render_wgpu::UiOverlayNode> {
    if node.id.as_deref() == Some(id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_overlay_node(child, id))
}

fn collect_overlay_ids(node: &amigo_render_wgpu::UiOverlayNode, ids: &mut Vec<String>) {
    if let Some(id) = &node.id {
        ids.push(id.clone());
    }
    for child in &node.children {
        collect_overlay_ids(child, ids);
    }
}

#[test]
fn layered_image_bounds_use_component_size_and_entity_translation() {
    let entity = test_node(
        "entity-bg",
        "background",
        amigo_editor_authoring::AuthoringNodeKind::Entity,
        "/entities/0",
        r#"
name: background
transform2:
  translation: { x: 40.0, y: 60.0 }
"#,
        Default::default(),
    );
    let component = test_node(
        "layered-bg",
        "LayeredImage2D",
        amigo_editor_authoring::AuthoringNodeKind::Component,
        "/entities/0/components/0",
        r#"
type: LayeredImage2D
size: { x: 320.0, y: 180.0 }
z_index: 2.0
"#,
        amigo_editor_authoring::AuthoringNodeSemantic {
            owner_entity_name: Some("background".to_owned()),
            component_type: Some("LayeredImage2D".to_owned()),
            ..Default::default()
        },
    );
    let graph = test_graph(vec![entity, component]);
    let state = state::IngameEditorState::new(true);

    assert!(crate::selection::select_viewport_target(
        &state, &graph, 50.0, 70.0
    ));
    assert_eq!(
        state.snapshot().selection.unwrap().logical_bounds,
        Some(state::EditorRect {
            x: 40.0,
            y: 60.0,
            width: 320.0,
            height: 180.0,
        })
    );
}

#[test]
fn particle_bounds_use_spawn_area_and_entity_translation() {
    let entity = test_node(
        "entity-rain",
        "rain-near",
        amigo_editor_authoring::AuthoringNodeKind::Entity,
        "/entities/0",
        r#"
name: rain-near
transform2:
  translation: { x: 500.0, y: 300.0 }
"#,
        Default::default(),
    );
    let component = test_node(
        "particle-rain",
        "ParticleEmitter2D",
        amigo_editor_authoring::AuthoringNodeKind::Component,
        "/entities/0/components/0",
        r#"
type: ParticleEmitter2D
spawn_area:
  size: { x: 200.0, y: 100.0 }
z_index: 5.0
"#,
        amigo_editor_authoring::AuthoringNodeSemantic {
            owner_entity_name: Some("rain-near".to_owned()),
            component_type: Some("ParticleEmitter2D".to_owned()),
            ..Default::default()
        },
    );
    let graph = test_graph(vec![entity, component]);
    let state = state::IngameEditorState::new(true);

    assert!(crate::selection::select_viewport_target(
        &state, &graph, 500.0, 300.0
    ));
    assert_eq!(
        state.snapshot().selection.unwrap().logical_bounds,
        Some(state::EditorRect {
            x: 400.0,
            y: 250.0,
            width: 200.0,
            height: 100.0,
        })
    );
}

#[test]
fn render_layer_property_panel_has_opacity_slider() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
id: background
label: Background
order: 0
visible: true
opacity: 0.8
"#,
    )
    .unwrap();

    let node = amigo_editor_authoring::AuthoringNode {
        id: "layer-node".to_owned(),
        label: "layer: background".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::RenderLayer,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::UseRef,
        source_file: "visual/render.yml".into(),
        yaml_pointer: "/visual2d/render_layers/0".to_owned(),
        editable: true,
        value,
        value_preview: "5 fields".to_owned(),
        semantic: Default::default(),
        children: Vec::new(),
    };

    let panel = crate::properties::build_panel_with_overrides(&node, |_| None);

    assert!(panel
        .groups
        .iter()
        .flat_map(|group| &group.properties)
        .any(|row| {
            row.label.eq_ignore_ascii_case("opacity")
                && crate::properties::is_slider(&row.editor).is_some()
        }));
}

#[test]
fn layered_image_uses_owner_entity_from_semantic_context() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
type: LayeredImage2D
asset: backgrounds/main_menu.yml
base_opacity: 1.0
"#,
    )
    .unwrap();

    let semantic = amigo_editor_authoring::AuthoringNodeSemantic {
        owner_entity_name: Some("main-menu-background".to_owned()),
        ..Default::default()
    };

    let node = amigo_editor_authoring::AuthoringNode {
        id: "component-node".to_owned(),
        label: "component: LayeredImage2D".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Component,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::UseRef,
        source_file: "entities/background.yml".into(),
        yaml_pointer: "/entities/0/components/0".to_owned(),
        editable: true,
        value,
        value_preview: "3 fields".to_owned(),
        semantic,
        children: Vec::new(),
    };

    let panel = crate::properties::build_panel_with_overrides(&node, |_| None);

    assert!(panel
        .groups
        .iter()
        .flat_map(|group| &group.properties)
        .any(|row| {
            row.label == "entity"
                && row.value
                    == amigo_editor_authoring::AuthoringPropertyValue::Text(
                        "main-menu-background".to_owned(),
                    )
        }));
}

#[test]
fn editor_layout_places_three_main_panels() {
    let layout =
        crate::layout::EditorLayout::new(amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0));
    assert!(layout.left_panel.rect.width > 0.0);
    assert!(layout.center_panel.rect.width > 0.0);
    assert!(layout.right_panel.rect.width > 0.0);
    assert!(layout.left_panel.rect.x < layout.center_panel.rect.x);
    assert!(layout.center_panel.rect.x < layout.right_panel.rect.x);
}

#[test]
fn editor_layout_fits_game_viewport_inside_center_panel() {
    let layout =
        crate::layout::EditorLayout::new(amigo_render_wgpu::UiViewportSize::new(1920.0, 1080.0));
    let game = layout.game_viewport_rect();

    assert!(game.x >= layout.center_panel.content_rect.x);
    assert!(game.y >= layout.center_panel.content_rect.y);
    assert!(
        game.x + game.width
            <= layout.center_panel.content_rect.x + layout.center_panel.content_rect.width
    );
    assert!(
        game.y + game.height
            <= layout.center_panel.content_rect.y + layout.center_panel.content_rect.height
    );
    assert!((game.width / game.height - 16.0 / 9.0).abs() < 0.001);
}

#[test]
fn editor_layout_maps_screen_points_to_logical_game_viewport() {
    let layout = layout::EditorLayout::new(amigo_render_wgpu::UiViewportSize::new(1920.0, 1080.0));
    let game = layout.game_viewport_layout();
    let center_x = game.rect.x + game.rect.width * 0.5;
    let center_y = game.rect.y + game.rect.height * 0.5;

    let logical = game
        .screen_to_logical(center_x, center_y)
        .expect("center point inside game viewport");

    assert!((logical.0 - layout::GAME_VIEWPORT_LOGICAL_W * 0.5).abs() < 0.1);
    assert!((logical.1 - layout::GAME_VIEWPORT_LOGICAL_H * 0.5).abs() < 0.1);
}

#[test]
fn editor_state_records_viewport_selection() {
    let state = IngameEditorState::new(true);

    state.select_scene_node(crate::state::EditorSelection {
        node_id: "node".to_owned(),
        source: crate::state::SelectionSource::Viewport,
        source_path: Some("scene.yml".to_owned()),
        yaml_pointer: Some("/entities/0".to_owned()),
        label: Some("background".to_owned()),
        logical_x: Some(640.0),
        logical_y: Some(360.0),
        logical_bounds: Some(state::EditorRect {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 720.0,
        }),
    });

    let snapshot = state.snapshot();
    assert_eq!(snapshot.selection.as_ref().map(|s| s.node_id.as_str()), Some("node"));
    assert_eq!(
        snapshot.selection.as_ref().and_then(|selection| selection.label.as_deref()),
        Some("background")
    );
}

#[test]
fn editor_layout_detects_tree_and_properties_panels() {
    let layout =
        crate::layout::EditorLayout::new(amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0));
    assert_eq!(
        layout.panel_for_point(
            layout.left_panel.rect.x + 4.0,
            layout.left_panel.rect.y + 4.0
        ),
        crate::layout::EditorPanelKind::Tree
    );
    assert_eq!(
        layout.panel_for_point(
            layout.right_panel.rect.x + 4.0,
            layout.right_panel.rect.y + 4.0
        ),
        crate::layout::EditorPanelKind::Properties
    );
}

#[test]
fn editor_layout_row_rects_stay_inside_panel_content() {
    let layout =
        crate::layout::EditorLayout::new(amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0));
    let tree_row = layout.tree_row_rect(1, layout.left_panel.content_rect.y + 44.0);
    assert!(tree_row.x >= layout.left_panel.content_rect.x);
    assert!(tree_row.width <= layout.left_panel.content_rect.width);
    let property_row = layout.property_row_rect(layout.right_panel.content_rect.y + 44.0);
    assert_eq!(property_row.x, layout.right_panel.content_rect.x);
    assert_eq!(property_row.width, layout.right_panel.content_rect.width);
}

#[test]
fn editor_state_expand_all_clears_collapsed_nodes() {
    let state = IngameEditorState::new(true);

    state.toggle_node_collapsed("a");
    state.toggle_node_collapsed("b");
    assert_eq!(state.collapsed_node_count(), 2);

    state.expand_all();

    assert_eq!(state.collapsed_node_count(), 0);
    assert!(!state.is_node_collapsed("a"));
    assert!(!state.is_node_collapsed("b"));
}

#[test]
fn editor_state_collapse_all_replaces_collapsed_nodes() {
    let state = IngameEditorState::new(true);

    state.toggle_node_collapsed("old");
    state.collapse_all(vec!["a".to_owned(), "b".to_owned()]);

    assert_eq!(state.collapsed_node_count(), 2);
    assert!(!state.is_node_collapsed("old"));
    assert!(state.is_node_collapsed("a"));
    assert!(state.is_node_collapsed("b"));
}

#[test]
fn editor_properties_display_new_authoring_value_variants() {
    assert_eq!(
        crate::properties::display_text(
            "asset",
            &amigo_editor_authoring::AuthoringPropertyValue::AssetRef("club-bg".to_owned()),
        ),
        "asset: asset:club-bg"
    );
    assert_eq!(
        crate::properties::display_text(
            "size",
            &amigo_editor_authoring::AuthoringPropertyValue::Vec2(1280.0, 720.0),
        ),
        "size: (1280.000, 720.000)"
    );
}

#[test]
fn editor_state_sets_and_clears_tree_filter() {
    let state = IngameEditorState::new(true);
    state.set_tree_filter(" rain ");
    let snapshot = state.snapshot();
    assert_eq!(snapshot.tree_filter, "rain");
    assert_eq!(snapshot.tree_scroll, 0.0);
    state.clear_tree_filter();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.tree_filter, "");
}

#[test]
fn editor_state_tree_filter_resets_scroll() {
    let state = IngameEditorState::new(true);
    state.set_scroll_bounds(100.0, 100.0);
    state.scroll_tree(80.0);
    assert_eq!(state.snapshot().tree_scroll, 80.0);
    state.set_tree_filter("LayeredImage2D");
    assert_eq!(state.snapshot().tree_scroll, 0.0);
}

#[test]
fn tree_row_label_uses_ascii_markers() {
    let child = amigo_editor_authoring::AuthoringNode {
        id: "child".to_owned(),
        label: "child".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Scalar,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent/child".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "null".to_owned(),
        semantic: Default::default(),
        children: Vec::new(),
    };
    let parent = amigo_editor_authoring::AuthoringNode {
        id: "parent".to_owned(),
        label: "parent".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Mapping,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "null".to_owned(),
        semantic: Default::default(),
        children: vec![child],
    };

    let graph = test_graph(vec![parent]);
    let projection = amigo_editor_authoring::raw_yaml_projection(&graph);
    let row = projection
        .rows
        .iter()
        .find(|row| row.node_id == "parent")
        .expect("parent row");
    assert_eq!(crate::theme::icon_ascii(row.icon), "[MAP]");
    assert!(row.tags.iter().all(|tag| tag.label != "Edit" && tag.label != "RO"));
}

#[test]
fn clean_tree_does_not_render_scalar_nodes() {
    let scalar = test_node(
        "scalar",
        "value",
        amigo_editor_authoring::AuthoringNodeKind::Scalar,
        "/value",
        "1",
        Default::default(),
    );
    let entity = test_node(
        "entity",
        "entity: hero",
        amigo_editor_authoring::AuthoringNodeKind::Entity,
        "/entities/0",
        "name: hero",
        Default::default(),
    );
    let component = test_node(
        "component",
        "component: Sprite2D",
        amigo_editor_authoring::AuthoringNodeKind::Component,
        "/entities/0/components/0",
        "type: Sprite2D",
        amigo_editor_authoring::AuthoringNodeSemantic {
            component_type: Some("Sprite2D".to_owned()),
            ..Default::default()
        },
    );
    let graph = test_graph(vec![scalar, entity, component]);
    let projection = amigo_editor_authoring::scene_objects_projection(&graph);
    assert!(!projection.rows.iter().any(|row| row.node_id == "scalar"));
    assert!(projection.rows.iter().any(|row| row.node_id == "entity"));
    assert!(projection.rows.iter().any(|row| row.node_id == "component"));
}

#[test]
fn primary_inspector_hides_hidden_properties() {
    let row = amigo_editor_authoring::AuthoringProperty {
        id: "hidden".to_owned(),
        label: "hidden".to_owned(),
        value: amigo_editor_authoring::AuthoringPropertyValue::Text("x".to_owned()),
        editor: amigo_editor_authoring::AuthoringPropertyEditor::ReadOnly,
        hints: amigo_editor_authoring::AuthoringPropertyHints::default(),
        read_only: true,
        source_file: "scene.yml".to_owned(),
        yaml_pointer: "/hidden".to_owned(),
        group: "debug".to_owned(),
        trait_kind: None,
        binding: None,
        display: amigo_editor_authoring::AuthoringPropertyDisplay {
            visibility: amigo_editor_authoring::AuthoringPropertyVisibility::Hidden,
            ..Default::default()
        },
    };
    let panel = amigo_editor_authoring::AuthoringPropertyPanel {
        title: "test".to_owned(),
        groups: vec![amigo_editor_authoring::AuthoringPropertyGroup {
            id: "debug".to_owned(),
            title: "Debug".to_owned(),
            properties: vec![row],
        }],
    };
    let filtered = amigo_editor_authoring::filter_property_panel_for_view(
        panel,
        amigo_editor_authoring::InspectorViewMode::Primary,
    );
    assert!(filtered.groups.is_empty());
}

#[test]
fn render_stack_collects_entities_by_render_layer() {
    let layer = test_node(
        "layer",
        "layer: midground",
        amigo_editor_authoring::AuthoringNodeKind::RenderLayer,
        "/visual2d/render_layers/0",
        "id: midground\norder: 2\nvisible: true\nopacity: 1.0\n",
        Default::default(),
    );
    let component = test_node(
        "component",
        "component: Sprite2D",
        amigo_editor_authoring::AuthoringNodeKind::Component,
        "/entities/0/components/0",
        "type: Sprite2D\nrender_layer: midground\n",
        amigo_editor_authoring::AuthoringNodeSemantic {
            component_type: Some("Sprite2D".to_owned()),
            ..Default::default()
        },
    );
    let entity = amigo_editor_authoring::AuthoringNode {
        id: "entity".to_owned(),
        label: "entity: hero".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Entity,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/entities/0".to_owned(),
        editable: true,
        value: serde_yaml::from_str("name: hero").unwrap(),
        value_preview: "1 field".to_owned(),
        semantic: Default::default(),
        children: vec![component],
    };
    let graph = test_graph(vec![layer, entity]);
    let layers = crate::overlay::collect_render_stack(&graph);
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].id, "midground");
    assert_eq!(layers[0].entities.len(), 1);
    assert_eq!(layers[0].entities[0].label, "hero");
}

#[test]
fn editor_overlay_clean_mode_shows_scene_objects_title() {
    let graph = test_graph(Vec::new());
    let state = IngameEditorState::new(true);
    state.set_tree_mode(crate::state::EditorTreeMode::Scene);
    let mut hit_targets = Vec::new();
    let mut stats = crate::overlay::OverlayStats::default();
    let document = crate::overlay::build_editor_document(
        amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0),
        &graph,
        None,
        "ok",
        &state,
        &mut hit_targets,
        &mut stats,
    );
    assert_eq!(
        find_overlay_text(&document.root, "editor-tree-title"),
        Some("SCENE GRAPH")
    );
}

#[test]
fn editor_overlay_tree_rows_use_ascii_icon_labels() {
    let entity = test_node(
        "entity-bg",
        "background",
        amigo_editor_authoring::AuthoringNodeKind::Entity,
        "/entities/0",
        "id: background\nname: Background",
        Default::default(),
    );
    let graph = test_graph(vec![entity]);
    let state = IngameEditorState::new(true);
    let mut hit_targets = Vec::new();
    let mut stats = crate::overlay::OverlayStats::default();
    let document = crate::overlay::build_editor_document(
        amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0),
        &graph,
        None,
        "ok",
        &state,
        &mut hit_targets,
        &mut stats,
    );

    let label = find_overlay_text(&document.root, "label:entity-bg").unwrap_or("");
    assert!(label.starts_with("[ENT]"));
}

#[test]
fn editor_overlay_render_stack_tab_shows_expected_titles() {
    let graph = test_graph(Vec::new());
    let state = IngameEditorState::new(true);
    state.set_tree_mode(crate::state::EditorTreeMode::Stack);
    let mut hit_targets = Vec::new();
    let mut stats = crate::overlay::OverlayStats::default();
    let document = crate::overlay::build_editor_document(
        amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0),
        &graph,
        None,
        "ok",
        &state,
        &mut hit_targets,
        &mut stats,
    );
    assert_eq!(
        find_overlay_text(&document.root, "editor-tree-title"),
        Some("RENDER STACK")
    );
}

#[test]
fn editor_overlay_scalar_selection_explains_raw_debug_only() {
    let scalar = test_node(
        "scalar-node",
        "scalar",
        amigo_editor_authoring::AuthoringNodeKind::Scalar,
        "/debug/value",
        "value",
        Default::default(),
    );
    let graph = test_graph(vec![scalar.clone()]);
    let state = IngameEditorState::new(true);
    let mut hit_targets = Vec::new();
    let mut stats = crate::overlay::OverlayStats::default();
    let document = crate::overlay::build_editor_document(
        amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0),
        &graph,
        Some(&scalar),
        "ok",
        &state,
        &mut hit_targets,
        &mut stats,
    );

    assert_eq!(
        find_overlay_text(&document.root, "editor-properties-title"),
        Some("Raw Debug: scalar")
    );
    assert!(find_overlay_text(&document.root, "editor-properties-title").is_some());
}

#[test]
fn tree_visibility_respects_collapse_but_filter_forces_matching_descendants_visible() {
    let child = amigo_editor_authoring::AuthoringNode {
        id: "child".to_owned(),
        label: "rain child".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Scalar,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent/child".to_owned(),
        editable: true,
        value: serde_yaml::Value::String("rain".to_owned()),
        value_preview: "rain".to_owned(),
        semantic: Default::default(),
        children: Vec::new(),
    };
    let parent = amigo_editor_authoring::AuthoringNode {
        id: "parent".to_owned(),
        label: "parent".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::Mapping,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "mapping".to_owned(),
        semantic: Default::default(),
        children: vec![child],
    };
    let graph = amigo_editor_authoring::AuthoringSceneGraph {
        source_mod: "test".to_owned(),
        scene_id: "test".to_owned(),
        root_file: "scene.yml".into(),
        source_files: vec!["scene.yml".into()],
        nodes: vec![parent],
    };

    let mut collapsed = std::collections::BTreeSet::new();
    collapsed.insert("parent".to_owned());

    assert!(crate::overlay::is_tree_node_visible(
        &graph, "parent", "", &collapsed
    ));
    assert!(!crate::overlay::is_tree_node_visible(
        &graph, "child", "", &collapsed
    ));
    assert!(crate::overlay::is_tree_node_visible(
        &graph, "child", "rain", &collapsed
    ));
}

#[test]
fn editor_hit_target_sync_drops_missing_overlay_nodes() {
    let viewport = amigo_render_wgpu::UiViewportSize::new(320.0, 200.0);
    let document = amigo_render_wgpu::UiOverlayDocument {
        entity_name: "test-editor".to_owned(),
        layer: amigo_render_wgpu::UiOverlayLayer::Debug,
        viewport: Some(amigo_render_wgpu::UiOverlayViewport {
            width: viewport.width,
            height: viewport.height,
            scaling: amigo_render_wgpu::UiOverlayViewportScaling::Expand,
        }),
        root: amigo_render_wgpu::UiOverlayNode {
            id: Some("root".to_owned()),
            kind: amigo_render_wgpu::UiOverlayNodeKind::Stack,
            style: amigo_render_wgpu::UiOverlayStyle {
                width: Some(viewport.width),
                height: Some(viewport.height),
                ..amigo_render_wgpu::UiOverlayStyle::default()
            },
            children: vec![amigo_render_wgpu::UiOverlayNode {
                id: Some("existing-control".to_owned()),
                kind: amigo_render_wgpu::UiOverlayNodeKind::Button {
                    text: "ok".to_owned(),
                    font: None,
                },
                style: amigo_render_wgpu::UiOverlayStyle {
                    left: Some(10.0),
                    top: Some(20.0),
                    width: Some(100.0),
                    height: Some(24.0),
                    ..amigo_render_wgpu::UiOverlayStyle::default()
                },
                children: Vec::new(),
            }],
        },
    };

    let mut hit_targets = vec![
        crate::state::EditorHitTarget {
            id: "existing-control".to_owned(),
            rect: crate::state::EditorRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            action: crate::state::EditorHitAction::ConsumeOnly,
        },
        crate::state::EditorHitTarget {
            id: "missing-control".to_owned(),
            rect: crate::state::EditorRect {
                x: 1.0,
                y: 1.0,
                width: 1.0,
                height: 1.0,
            },
            action: crate::state::EditorHitAction::ConsumeOnly,
        },
    ];

    crate::overlay::sync_hit_targets_from_layout(viewport, &document, &mut hit_targets);

    assert_eq!(hit_targets.len(), 1);
    assert_eq!(hit_targets[0].id, "existing-control");
    assert_eq!(hit_targets[0].rect.x, 10.0);
    assert_eq!(hit_targets[0].rect.y, 20.0);
}

#[test]
fn tree_filter_matches_light_semantic_fields() {
    let light = amigo_editor_authoring::AuthoringNode {
        id: "light".to_owned(),
        label: "light".to_owned(),
        kind: amigo_editor_authoring::AuthoringNodeKind::LightGroup,
        origin: amigo_editor_authoring::AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/visual2d/light_groups/0".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "light".to_owned(),
        semantic: amigo_editor_authoring::AuthoringNodeSemantic {
            light_group_id: Some("skyline".to_owned()),
            ..Default::default()
        },
        children: Vec::new(),
    };
    let graph = amigo_editor_authoring::AuthoringSceneGraph {
        source_mod: "test".to_owned(),
        scene_id: "test".to_owned(),
        root_file: "scene.yml".into(),
        source_files: vec!["scene.yml".into()],
        nodes: vec![light],
    };
    assert!(crate::overlay::is_tree_node_visible(
        &graph,
        "light",
        "skyline",
        &std::collections::BTreeSet::new(),
    ));
}

fn test_property_for_hit_target(
    editor: amigo_editor_authoring::AuthoringPropertyEditor,
    apply_mode: amigo_editor_authoring::AuthoringPropertyApplyMode,
    read_only: bool,
    binding: Option<amigo_editor_authoring::AuthoringRuntimeBinding>,
) -> amigo_editor_authoring::AuthoringProperty {
    amigo_editor_authoring::AuthoringProperty {
        id: "prop".to_owned(),
        label: "Prop".to_owned(),
        value: amigo_editor_authoring::AuthoringPropertyValue::Number(1.0),
        editor,
        hints: amigo_editor_authoring::AuthoringPropertyHints::default(),
        read_only,
        source_file: "scene.yml".to_owned(),
        yaml_pointer: "/prop".to_owned(),
        group: "test".to_owned(),
        trait_kind: None,
        binding,
        display: amigo_editor_authoring::AuthoringPropertyDisplay {
            apply_mode,
            ..Default::default()
        },
    }
}

#[test]
fn theme_icon_labels_are_ascii_and_glyphs_are_real_icons() {
    let label = crate::theme::icon_ascii(amigo_editor_authoring::AuthoringTreeIcon::Image);
    let glyph = crate::theme::icon_glyph(amigo_editor_authoring::AuthoringTreeIcon::Image);
    assert_eq!(label, "[IMG]");
    assert!(label.is_ascii());
    assert_eq!(glyph, "\u{f03e}");
}

#[test]
fn editor_icon_font_asset_is_registered_and_queued() {
    let runtime = amigo_runtime::RuntimeBuilder::default()
        .with_service(amigo_assets::AssetCatalog::default())
        .expect("asset catalog service")
        .build();

    crate::overlay::ensure_editor_icon_font_asset(&runtime);

    let assets = runtime
        .resolve::<amigo_assets::AssetCatalog>()
        .expect("asset catalog");
    let key = crate::theme::editor_icon_font();
    assert!(assets.contains(&key));
    assert!(assets
        .pending_loads()
        .iter()
        .any(|request| request.key == key));
    let manifest = assets.manifest(&key).expect("editor icon font manifest");
    assert_eq!(
        manifest.source,
        amigo_assets::AssetSourceKind::Mod("core".to_owned())
    );
}

#[test]
fn unsupported_and_readonly_properties_have_no_hit_target() {
    let binding = Some(
        amigo_editor_authoring::AuthoringRuntimeBinding::RenderLayerOpacity {
            layer_id: "midground".to_owned(),
        },
    );
    let unsupported = test_property_for_hit_target(
        amigo_editor_authoring::AuthoringPropertyEditor::Number,
        amigo_editor_authoring::AuthoringPropertyApplyMode::Unsupported,
        false,
        binding.clone(),
    );
    let readonly = test_property_for_hit_target(
        amigo_editor_authoring::AuthoringPropertyEditor::Number,
        amigo_editor_authoring::AuthoringPropertyApplyMode::Live,
        true,
        binding,
    );
    assert!(!crate::overlay::property_has_editable_hit_target(
        &unsupported
    ));
    assert!(!crate::overlay::property_has_editable_hit_target(&readonly));
}

#[test]
fn only_supported_generic_editors_have_hit_targets() {
    let binding = Some(
        amigo_editor_authoring::AuthoringRuntimeBinding::RenderLayerOpacity {
            layer_id: "midground".to_owned(),
        },
    );
    let number = test_property_for_hit_target(
        amigo_editor_authoring::AuthoringPropertyEditor::Number,
        amigo_editor_authoring::AuthoringPropertyApplyMode::Live,
        false,
        binding.clone(),
    );
    let text = test_property_for_hit_target(
        amigo_editor_authoring::AuthoringPropertyEditor::Text,
        amigo_editor_authoring::AuthoringPropertyApplyMode::Live,
        false,
        binding.clone(),
    );
    let asset = test_property_for_hit_target(
        amigo_editor_authoring::AuthoringPropertyEditor::AssetPicker {
            domain: "LayeredImage".to_owned(),
        },
        amigo_editor_authoring::AuthoringPropertyApplyMode::Live,
        false,
        binding,
    );
    assert!(crate::overlay::property_has_editable_hit_target(&number));
    assert!(!crate::overlay::property_has_editable_hit_target(&text));
    assert!(!crate::overlay::property_has_editable_hit_target(&asset));
}

#[test]
fn particle_bounds_use_descriptor_policy_without_component_exception() {
    let registry = amigo_scene::default_component_registry();
    let descriptor = registry
        .descriptor_by_type_name("ParticleEmitter2D")
        .expect("ParticleEmitter2D descriptor");
    assert!(matches!(
        descriptor.bounds_policy,
        amigo_scene::BoundsPolicy::SpawnArea2D { .. }
    ));
}
