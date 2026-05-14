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
        state.snapshot().viewport_selection.unwrap().logical_bounds,
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
        state.snapshot().viewport_selection.unwrap().logical_bounds,
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

    assert!(
        panel
            .groups
            .iter()
            .flat_map(|group| &group.properties)
            .any(|row| {
                row.label == "opacity" && crate::properties::is_slider(&row.editor).is_some()
            })
    );
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

    assert!(
        panel
            .groups
            .iter()
            .flat_map(|group| &group.properties)
            .any(|row| {
                row.label == "entity"
                    && row.value
                        == amigo_editor_authoring::AuthoringPropertyValue::Text(
                            "main-menu-background".to_owned(),
                        )
            })
    );
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

    state.select_viewport_node(
        "node",
        Some("scene.yml".to_owned()),
        Some("/entities/0".to_owned()),
        Some("background".to_owned()),
        Some("LayeredImage2D".to_owned()),
        640.0,
        360.0,
        Some(state::EditorRect {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 720.0,
        }),
    );

    let snapshot = state.snapshot();
    assert_eq!(snapshot.selected_node_id.as_deref(), Some("node"));
    assert_eq!(
        snapshot
            .viewport_selection
            .as_ref()
            .and_then(|selection| selection.entity_name.as_deref()),
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
fn tree_row_label_uses_clean_unicode_twisties() {
    let state = IngameEditorState::new(true);
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

    let expanded = crate::overlay::tree_row_label(&parent, &state);
    assert!(expanded.starts_with("\u{25BE} "));
    assert!(!expanded.contains("â"));

    state.toggle_node_collapsed("parent");
    let collapsed = crate::overlay::tree_row_label(&parent, &state);
    assert!(collapsed.starts_with("\u{25B8} "));
    assert!(!collapsed.contains("â"));
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
