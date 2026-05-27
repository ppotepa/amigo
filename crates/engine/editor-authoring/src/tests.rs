use super::*;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn authoring_service_starts_without_cached_scenes() {
    let service = AuthoringSceneGraphService::default();
    assert_eq!(service.cached_scene_count(), 0);
}

#[test]
fn authoring_node_find_walks_children() {
    let child = AuthoringNode {
        id: "child".to_owned(),
        label: "child".to_owned(),
        kind: AuthoringNodeKind::Scalar,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/child".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "null".to_owned(),
        semantic: AuthoringNodeSemantic::default(),
        children: Vec::new(),
    };
    let graph = AuthoringSceneGraph {
        source_mod: "mod".to_owned(),
        scene_id: "scene".to_owned(),
        root_file: "scene.yml".into(),
        source_files: vec!["scene.yml".into()],
        nodes: vec![AuthoringNode {
            id: "root".to_owned(),
            label: "root".to_owned(),
            kind: AuthoringNodeKind::File,
            origin: AuthoringNodeOrigin::Root,
            source_file: "scene.yml".into(),
            yaml_pointer: "/".to_owned(),
            editable: false,
            value: serde_yaml::Value::Null,
            value_preview: "null".to_owned(),
            semantic: AuthoringNodeSemantic::default(),
            children: vec![child],
        }],
    };

    assert_eq!(
        graph.find_node("child").map(|node| node.label.as_str()),
        Some("child")
    );
    assert_eq!(graph.first_editable_node_id().as_deref(), Some("child"));
}

#[test]
fn rotten_club_main_menu_loads_use_source_files() {
    let root = repo_root();
    let mod_root = root.join("mods/rotten-club");
    let scene_file = mod_root.join("scenes/main-menu/scene.yml");

    let graph = load_authoring_scene_graph_from_file(
        "rotten-club".to_owned(),
        "main-menu".to_owned(),
        &mod_root,
        scene_file,
    )
    .expect("authoring graph");

    assert!(
        graph
            .source_files
            .iter()
            .any(|path| path.ends_with("camera/main.yml"))
    );
    assert!(
        graph
            .source_files
            .iter()
            .any(|path| path.ends_with("camera/motion.yml"))
    );
    assert!(
        graph
            .source_files
            .iter()
            .any(|path| path.ends_with("render/layers.yml"))
    );
}

#[test]
fn rotten_club_main_menu_has_render_layer_nodes() {
    let root = repo_root();
    let mod_root = root.join("mods/rotten-club");
    let scene_file = mod_root.join("scenes/main-menu/scene.yml");

    let graph = load_authoring_scene_graph_from_file(
        "rotten-club".to_owned(),
        "main-menu".to_owned(),
        &mod_root,
        scene_file,
    )
    .expect("authoring graph");

    fn has_layer(nodes: &[AuthoringNode]) -> bool {
        nodes
            .iter()
            .any(|node| node.kind == AuthoringNodeKind::RenderLayer || has_layer(&node.children))
    }

    assert!(has_layer(&graph.nodes));
}

#[test]
fn rotten_club_main_menu_has_ui_nodes() {
    let graph = load_rotten_club_main_menu_graph();

    assert!(!nodes_by_kind(&graph, AuthoringNodeKind::UiNode).is_empty());
}

fn test_component_node(component_type: &str, entity_name: &str, yaml: &str) -> AuthoringNode {
    AuthoringNode {
        id: "scene.yml#/entities/0/components/0".to_owned(),
        label: format!("component: {component_type}"),
        kind: AuthoringNodeKind::Component,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/entities/0/components/0".to_owned(),
        editable: true,
        value: serde_yaml::from_str(yaml).expect("component yaml"),
        value_preview: "component".to_owned(),
        semantic: AuthoringNodeSemantic {
            owner_entity_name: Some(entity_name.to_owned()),
            component_type: Some(component_type.to_owned()),
            ..AuthoringNodeSemantic::default()
        },
        children: Vec::new(),
    }
}

fn property_by_suffix<'a>(
    panel: &'a AuthoringPropertyPanel,
    suffix: &str,
) -> &'a AuthoringProperty {
    panel
        .groups
        .iter()
        .flat_map(|group| group.properties.iter())
        .find(|property| property.id.ends_with(suffix))
        .unwrap_or_else(|| panic!("missing property suffix `{suffix}`"))
}

#[test]
fn component_descriptor_generates_particle_emitter_properties_from_scene_metadata() {
    let node = test_component_node(
        "ParticleEmitter2D",
        "rain-near",
        r#"
type: ParticleEmitter2D
active: true
spawn_rate: 85
max_particles: 300
particle_lifetime: 1.2
initial_speed: 640
render_layer: rain.near
"#,
    );

    let panel = build_property_panel_for_node(&node);

    let spawn_rate = property_by_suffix(&panel, "::spawn_rate");
    assert_eq!(spawn_rate.label, "Spawn Rate");
    assert_eq!(spawn_rate.group, "render2d.content");
    assert!(matches!(
        spawn_rate.editor,
        AuthoringPropertyEditor::Slider { .. }
    ));
    assert_eq!(
        spawn_rate.binding,
        Some(AuthoringRuntimeBinding::ParticleEmitterProperty {
            entity_name: "rain-near".to_owned(),
            field: "spawn_rate".to_owned(),
        })
    );

    let active = property_by_suffix(&panel, "::active");
    assert_eq!(active.label, "Active");
    assert!(matches!(active.editor, AuthoringPropertyEditor::Toggle));
}

#[test]
fn layered_image_uses_descriptor_and_preserves_dynamic_layer_override_bindings() {
    let node = test_component_node(
        "LayeredImage2D",
        "background",
        r#"
type: LayeredImage2D
render_layer: background.city
asset: club-bg
base_opacity: 0.8
z_index: 0
layer_overrides:
  - id: background.city
    opacity: 0.35
    enabled: false
"#,
    );

    let panel = build_property_panel_for_node(&node);

    let asset = property_by_suffix(&panel, "::asset");
    assert_eq!(asset.label, "Layered Image Asset");

    let base_opacity = property_by_suffix(&panel, "::base_opacity");
    assert_eq!(base_opacity.label, "Base Opacity");
    assert!(matches!(
        base_opacity.editor,
        AuthoringPropertyEditor::Slider { .. }
    ));
    assert_eq!(
        base_opacity.binding,
        Some(AuthoringRuntimeBinding::LayeredImageBaseOpacity {
            entity_name: "background".to_owned(),
        })
    );

    let override_opacity = property_by_suffix(&panel, "::layer_overrides.background.city.opacity");
    assert_eq!(
        override_opacity.binding,
        Some(AuthoringRuntimeBinding::LayeredImageLayerOpacity {
            entity_name: "background".to_owned(),
            layer_id: "background.city".to_owned(),
        })
    );

    let override_enabled = property_by_suffix(&panel, "::layer_overrides.background.city.enabled");
    assert_eq!(
        override_enabled.binding,
        Some(AuthoringRuntimeBinding::LayeredImageLayerEnabled {
            entity_name: "background".to_owned(),
            layer_id: "background.city".to_owned(),
        })
    );
}

#[test]
fn component_descriptor_panel_does_not_fallback_to_generic_yaml_when_descriptor_exists() {
    let node = test_component_node(
        "ParticleEmitter2D",
        "rain-near",
        r#"
type: ParticleEmitter2D
active: true
spawn_rate: 85
render_layer: rain.near
"#,
    );

    let panel = build_property_panel_for_node(&node);

    assert_eq!(panel.title, "Component: Particle Emitter 2D");
    assert!(panel.groups.iter().any(|group| group.id == "metadata"));
    assert!(
        panel
            .groups
            .iter()
            .any(|group| group.id == "render2d.content")
    );
    assert!(!panel.groups.iter().any(|group| group.id == "yaml"));
}

#[test]
fn entity_panel_includes_descriptor_backed_component_properties() {
    let component = test_component_node(
        "LayeredImage2D",
        "background",
        r#"
type: LayeredImage2D
asset: club-bg
base_opacity: 0.8
"#,
    );
    let entity = AuthoringNode {
        id: "scene.yml#/entities/0".to_owned(),
        label: "entity: background".to_owned(),
        kind: AuthoringNodeKind::Entity,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/entities/0".to_owned(),
        editable: true,
        value: serde_yaml::from_str("name: background").expect("entity yaml"),
        value_preview: "entity".to_owned(),
        semantic: AuthoringNodeSemantic::default(),
        children: vec![component],
    };

    let panel = build_property_panel_for_node(&entity);

    assert!(panel.groups.iter().any(|group| group.title == "Components"));
    assert!(
        panel
            .groups
            .iter()
            .flat_map(|group| group.properties.iter())
            .any(|property| property.label == "LayeredImage2D")
    );
}

#[test]
fn layered_image_descriptor_preserves_asset_ref_and_vec2_values() {
    let node = test_component_node(
        "LayeredImage2D",
        "background",
        r#"
type: LayeredImage2D
render_layer: background.city
asset: club-bg
size: { x: 1280.0, y: 720.0 }
base_opacity: 0.8
z_index: 0
layer_overrides: []
"#,
    );

    let panel = build_property_panel_for_node(&node);

    let asset = property_by_suffix(&panel, "::asset");
    assert_eq!(
        asset.value,
        AuthoringPropertyValue::AssetRef("club-bg".to_owned())
    );

    let size = property_by_suffix(&panel, "::size");
    assert_eq!(size.value, AuthoringPropertyValue::Vec2(1280.0, 720.0));
}

#[test]
fn layered_image_descriptor_provides_number_constraints_from_scene_metadata() {
    let node = test_component_node(
        "LayeredImage2D",
        "background",
        r#"
type: LayeredImage2D
render_layer: background.city
asset: club-bg
base_opacity: 0.8
z_index: 0
layer_overrides: []
"#,
    );

    let panel = build_property_panel_for_node(&node);
    let base_opacity = property_by_suffix(&panel, "::base_opacity");

    let constraints = base_opacity
        .hints
        .number
        .as_ref()
        .expect("base_opacity number constraints");
    assert_eq!(constraints.min, Some(0.0));
    assert_eq!(constraints.max, Some(1.0));
    assert_eq!(constraints.step, Some(0.01));
}

#[test]
fn particle_descriptor_provides_slider_constraints_from_scene_metadata() {
    let node = test_component_node(
        "ParticleEmitter2D",
        "rain-near",
        r#"
type: ParticleEmitter2D
active: true
spawn_rate: 85
initial_speed: 640
"#,
    );

    let panel = build_property_panel_for_node(&node);
    let spawn_rate = property_by_suffix(&panel, "::spawn_rate");

    let constraints = spawn_rate
        .hints
        .number
        .as_ref()
        .expect("spawn_rate number constraints");
    assert_eq!(constraints.min, Some(0.0));
    assert_eq!(constraints.max, Some(1000.0));
    assert_eq!(constraints.step, Some(1.0));
}

#[test]
fn layered_image_dynamic_property_id_keeps_dotted_layer_id() {
    let node = test_component_node(
        "LayeredImage2D",
        "background",
        r#"
type: LayeredImage2D
render_layer: background.city
asset: club-bg
base_opacity: 0.8
layer_overrides:
  - id: background.city.neon
    opacity: 0.25
    enabled: true
"#,
    );

    let panel = build_property_panel_for_node(&node);

    let opacity = property_by_suffix(&panel, "::layer_overrides.background.city.neon.opacity");
    assert_eq!(
        opacity.binding,
        Some(AuthoringRuntimeBinding::LayeredImageLayerOpacity {
            entity_name: "background".to_owned(),
            layer_id: "background.city.neon".to_owned(),
        })
    );
}

fn test_yaml_node(
    id: &str,
    label: &str,
    kind: AuthoringNodeKind,
    yaml_pointer: &str,
    yaml: &str,
    semantic: AuthoringNodeSemantic,
) -> AuthoringNode {
    AuthoringNode {
        id: id.to_owned(),
        label: label.to_owned(),
        kind,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: yaml_pointer.to_owned(),
        editable: true,
        value: serde_yaml::from_str(yaml).expect("yaml node"),
        value_preview: "mapping".to_owned(),
        semantic,
        children: Vec::new(),
    }
}

fn sample_node_for_kind(kind: AuthoringNodeKind) -> AuthoringNode {
    let (yaml, semantic) = match kind {
        AuthoringNodeKind::Component => (
            "type: ParticleEmitter2D\nactive: true\nspawn_rate: 10\n",
            AuthoringNodeSemantic {
                owner_entity_name: Some("entity".to_owned()),
                component_type: Some("ParticleEmitter2D".to_owned()),
                ..Default::default()
            },
        ),
        AuthoringNodeKind::RenderLayer => (
            "id: background.city\nvisible: true\nopacity: 1.0\norder: 0\n",
            AuthoringNodeSemantic {
                render_layer_id: Some("background.city".to_owned()),
                ..Default::default()
            },
        ),
        AuthoringNodeKind::PostFxItem => (
            "id: fx\ntype: mock\nsurface: { blur_px: 2.0 }\n",
            AuthoringNodeSemantic {
                post_fx_id: Some("fx".to_owned()),
                post_fx_type: Some("mock".to_owned()),
                ..Default::default()
            },
        ),
        AuthoringNodeKind::LightGroup => (
            "id: skyline\nintensity: 1.0\n",
            AuthoringNodeSemantic {
                light_group_id: Some("skyline".to_owned()),
                ..Default::default()
            },
        ),
        AuthoringNodeKind::LightRoute => (
            "receiver_layer: weather.rain.near\nopacity: 1.0\n",
            AuthoringNodeSemantic {
                light_route_receiver_layer: Some("weather.rain.near".to_owned()),
                ..Default::default()
            },
        ),
        AuthoringNodeKind::PrefabRef => {
            ("prefabs/background.yml", AuthoringNodeSemantic::default())
        }
        AuthoringNodeKind::PrefabOverrides => ("opacity: 1.0\n", AuthoringNodeSemantic::default()),
        AuthoringNodeKind::Use => (
            "path: visual/render.yml\n",
            AuthoringNodeSemantic::default(),
        ),
        AuthoringNodeKind::Scalar => ("42", AuthoringNodeSemantic::default()),
        AuthoringNodeKind::Sequence => ("- one\n- two\n", AuthoringNodeSemantic::default()),
        _ => ("name: value\n", AuthoringNodeSemantic::default()),
    };

    test_yaml_node(
        "scene.yml#/sample",
        "sample",
        kind,
        "/sample",
        yaml,
        semantic,
    )
}

#[test]
fn every_authoring_node_kind_returns_non_empty_property_panel() {
    for kind in [
        AuthoringNodeKind::File,
        AuthoringNodeKind::Use,
        AuthoringNodeKind::Scene,
        AuthoringNodeKind::Visual2d,
        AuthoringNodeKind::RenderLayers,
        AuthoringNodeKind::RenderLayer,
        AuthoringNodeKind::LightGroups,
        AuthoringNodeKind::LightGroup,
        AuthoringNodeKind::LightRoutes,
        AuthoringNodeKind::LightRoute,
        AuthoringNodeKind::PostFx,
        AuthoringNodeKind::PostFxItem,
        AuthoringNodeKind::Entities,
        AuthoringNodeKind::Entity,
        AuthoringNodeKind::Components,
        AuthoringNodeKind::Component,
        AuthoringNodeKind::PrefabRef,
        AuthoringNodeKind::PrefabOverrides,
        AuthoringNodeKind::Mapping,
        AuthoringNodeKind::Sequence,
        AuthoringNodeKind::Scalar,
    ] {
        let node = sample_node_for_kind(kind.clone());
        let panel = build_property_panel_for_node(&node);
        assert!(!panel.title.trim().is_empty(), "{kind:?} missing title");
        assert!(
            panel
                .groups
                .iter()
                .any(|group| !group.properties.is_empty()),
            "{kind:?} returned empty inspector panel"
        );
    }
}

#[test]
fn light_group_panel_reports_readonly_status_without_mock_controls() {
    let node = test_yaml_node(
        "scene.yml#/visual2d/light_groups/0",
        "light: neon",
        AuthoringNodeKind::LightGroup,
        "/visual2d/light_groups/0",
        r##"
id: neon
intensity: 0.75
enabled: true
color: "#ff00cc"
"##,
        AuthoringNodeSemantic {
            light_group_id: Some("neon".to_owned()),
            ..AuthoringNodeSemantic::default()
        },
    );
    let panel = build_property_panel_for_node(&node);
    let status = property_by_suffix(&panel, "::status");
    assert!(status.read_only);
    assert!(matches!(status.editor, AuthoringPropertyEditor::ReadOnly));
    assert!(status.binding.is_none());
    assert_eq!(
        status.display.apply_mode,
        AuthoringPropertyApplyMode::ReadOnly
    );
}

#[test]
fn light_route_panel_reports_readonly_status() {
    let node = test_yaml_node(
        "scene.yml#/visual2d/light_routes/0",
        "route: background.city",
        AuthoringNodeKind::LightRoute,
        "/visual2d/light_routes/0",
        r#"
receiver_layer: background.city
light_group: neon
opacity: 0.5
"#,
        AuthoringNodeSemantic {
            light_route_receiver_layer: Some("background.city".to_owned()),
            ..AuthoringNodeSemantic::default()
        },
    );
    let panel = build_property_panel_for_node(&node);
    assert_eq!(panel.title, "Light Route: background.city");
    let status = property_by_suffix(&panel, "::status");
    assert!(status.read_only);
    assert!(status.binding.is_none());
}

#[test]
fn loader_classifies_light_group_and_route_sequence_items() {
    let root = repo_root();
    let temp_root = root.join("target/editor-authoring-test-light-kinds");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(temp_root.join("scenes/test")).expect("temp dirs");
    let scene_file = temp_root.join("scenes/test/scene.yml");
    std::fs::write(
        &scene_file,
        r##"
scene:
  id: test
visual2d:
  light_groups:
    - id: neon
      intensity: 1.0
  light_routes:
    - receiver_layer: background.city
      light_group: neon
"##,
    )
    .expect("write scene");
    let graph = load_authoring_scene_graph_from_file(
        "test".to_owned(),
        "test".to_owned(),
        &temp_root,
        scene_file,
    )
    .expect("authoring graph");
    fn has_kind(nodes: &[AuthoringNode], kind: &AuthoringNodeKind) -> bool {
        nodes
            .iter()
            .any(|node| &node.kind == kind || has_kind(&node.children, kind))
    }
    assert!(has_kind(&graph.nodes, &AuthoringNodeKind::LightGroup));
    assert!(has_kind(&graph.nodes, &AuthoringNodeKind::LightRoute));
}

#[test]
fn authoring_graph_builds_breadcrumb_for_nested_node() {
    let grandchild = AuthoringNode {
        id: "grandchild".to_owned(),
        label: "grandchild".to_owned(),
        kind: AuthoringNodeKind::Scalar,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent/child/grandchild".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "null".to_owned(),
        semantic: AuthoringNodeSemantic::default(),
        children: Vec::new(),
    };
    let child = AuthoringNode {
        id: "child".to_owned(),
        label: "child".to_owned(),
        kind: AuthoringNodeKind::Mapping,
        origin: AuthoringNodeOrigin::Root,
        source_file: "scene.yml".into(),
        yaml_pointer: "/parent/child".to_owned(),
        editable: true,
        value: serde_yaml::Value::Null,
        value_preview: "null".to_owned(),
        semantic: AuthoringNodeSemantic::default(),
        children: vec![grandchild],
    };
    let graph = AuthoringSceneGraph {
        source_mod: "mod".to_owned(),
        scene_id: "scene".to_owned(),
        root_file: "scene.yml".into(),
        source_files: vec!["scene.yml".into()],
        nodes: vec![AuthoringNode {
            id: "root".to_owned(),
            label: "root".to_owned(),
            kind: AuthoringNodeKind::File,
            origin: AuthoringNodeOrigin::Root,
            source_file: "scene.yml".into(),
            yaml_pointer: "/".to_owned(),
            editable: false,
            value: serde_yaml::Value::Null,
            value_preview: "null".to_owned(),
            semantic: AuthoringNodeSemantic::default(),
            children: vec![child],
        }],
    };
    assert_eq!(
        graph.breadcrumb_for_node("grandchild"),
        vec![
            "root".to_owned(),
            "child".to_owned(),
            "grandchild".to_owned()
        ]
    );
}

fn load_rotten_club_main_menu_graph() -> AuthoringSceneGraph {
    let root = repo_root();
    let mod_root = root.join("mods/rotten-club");
    let scene_file = mod_root.join("scenes/main-menu/scene.yml");
    load_authoring_scene_graph_from_file(
        "rotten-club".to_owned(),
        "main-menu".to_owned(),
        &mod_root,
        scene_file,
    )
    .expect("rotten-club/main-menu authoring graph")
}

fn collect_nodes_by_kind<'a>(
    nodes: &'a [AuthoringNode],
    kind: &AuthoringNodeKind,
    out: &mut Vec<&'a AuthoringNode>,
) {
    for node in nodes {
        if &node.kind == kind {
            out.push(node);
        }
        collect_nodes_by_kind(&node.children, kind, out);
    }
}

fn nodes_by_kind(graph: &AuthoringSceneGraph, kind: AuthoringNodeKind) -> Vec<&AuthoringNode> {
    let mut out = Vec::new();
    collect_nodes_by_kind(&graph.nodes, &kind, &mut out);
    out
}

fn first_component_by_type<'a>(
    graph: &'a AuthoringSceneGraph,
    component_type: &str,
) -> &'a AuthoringNode {
    nodes_by_kind(graph, AuthoringNodeKind::Component)
        .into_iter()
        .find(|node| node.semantic.component_type.as_deref() == Some(component_type))
        .unwrap_or_else(|| panic!("missing component `{component_type}`"))
}

fn first_render_layer_by_id<'a>(
    graph: &'a AuthoringSceneGraph,
    layer_id: &str,
) -> &'a AuthoringNode {
    nodes_by_kind(graph, AuthoringNodeKind::RenderLayer)
        .into_iter()
        .find(|node| node.semantic.render_layer_id.as_deref() == Some(layer_id))
        .unwrap_or_else(|| panic!("missing render layer `{layer_id}`"))
}

#[test]
fn rotten_club_main_menu_layered_image_has_owner_entity_and_bindings() {
    let graph = load_rotten_club_main_menu_graph();
    let node = first_component_by_type(&graph, "LayeredImage2D");
    assert_eq!(
        node.semantic.owner_entity_name.as_deref(),
        Some("background")
    );
    let panel = build_property_panel_for_node(node);
    let base_opacity = property_by_suffix(&panel, "::base_opacity");
    assert_eq!(
        base_opacity.binding,
        Some(AuthoringRuntimeBinding::LayeredImageBaseOpacity {
            entity_name: "background".to_owned(),
        })
    );
    let club_sign = property_by_suffix(&panel, "::layer_overrides.club_sign.opacity");
    assert_eq!(
        club_sign.binding,
        Some(AuthoringRuntimeBinding::LayeredImageLayerOpacity {
            entity_name: "background".to_owned(),
            layer_id: "club_sign".to_owned(),
        })
    );
}

#[test]
fn rotten_club_main_menu_render_layers_generate_runtime_bindings() {
    let graph = load_rotten_club_main_menu_graph();
    let node = first_render_layer_by_id(&graph, "background.city");
    let panel = build_property_panel_for_node(node);
    let opacity = property_by_suffix(&panel, "::opacity");
    assert_eq!(
        opacity.binding,
        Some(AuthoringRuntimeBinding::RenderLayerOpacity {
            layer_id: "background.city".to_owned(),
        })
    );
    let visible = property_by_suffix(&panel, "::visible");
    assert_eq!(
        visible.binding,
        Some(AuthoringRuntimeBinding::RenderLayerVisible {
            layer_id: "background.city".to_owned(),
        })
    );
    let order = property_by_suffix(&panel, "::order");
    assert_eq!(
        order.binding,
        Some(AuthoringRuntimeBinding::RenderLayerOrder {
            layer_id: "background.city".to_owned(),
        })
    );
    let depth_mode = property_by_suffix(&panel, "::depth.mode");
    assert_eq!(
        depth_mode.binding,
        Some(AuthoringRuntimeBinding::RenderLayerDepthMode {
            layer_id: "background.city".to_owned(),
        })
    );
    let option_ids = match &depth_mode.editor {
        AuthoringPropertyEditor::Enum { options } => {
            options.iter().map(String::as_str).collect::<Vec<_>>()
        }
        editor => panic!("depth.mode should use enum editor, got {editor:?}"),
    };
    assert!(option_ids.contains(&"distance"));
    assert!(option_ids.contains(&"infinity"));

    let rain_node = first_render_layer_by_id(&graph, "weather.rain.5m");
    let rain_panel = build_property_panel_for_node(rain_node);
    let distance_m = property_by_suffix(&rain_panel, "::depth.distance_m");
    assert_eq!(
        distance_m.binding,
        Some(AuthoringRuntimeBinding::RenderLayerDistanceM {
            layer_id: "weather.rain.5m".to_owned(),
        })
    );
    let z_depth = property_by_suffix(&rain_panel, "::depth.z_depth");
    assert_eq!(
        z_depth.binding,
        Some(AuthoringRuntimeBinding::RenderLayerZDepth {
            layer_id: "weather.rain.5m".to_owned(),
        })
    );
    let blur_scale = property_by_suffix(&rain_panel, "::depth.blur_scale");
    assert_eq!(
        blur_scale.binding,
        Some(AuthoringRuntimeBinding::RenderLayerDepthBlurScale {
            layer_id: "weather.rain.5m".to_owned(),
        })
    );
}

#[test]
fn rotten_club_main_menu_camera_reports_profile_refs_without_scene_postfx_mock_dump() {
    let graph = load_rotten_club_main_menu_graph();
    assert!(
        nodes_by_kind(&graph, AuthoringNodeKind::PostFxItem)
            .into_iter()
            .all(|node| node.semantic.post_fx_id.as_deref() != Some("rotten_shutter_history"))
    );

    let node = first_component_by_type(&graph, "Camera2D");
    assert_eq!(
        node.source_file
            .to_string_lossy()
            .replace('\\', "/")
            .ends_with("camera/main.yml"),
        true
    );
    assert!(
        node.value
            .get("lens_surface")
            .and_then(|value| value.get("rain_profile"))
            .and_then(|value| value.as_str())
            == Some("realistic_lens_rain")
    );
    let panel = build_property_panel_for_node(node);
    assert!(
        !panel
            .groups
            .iter()
            .flat_map(|group| group.properties.iter())
            .any(|property| property.id.ends_with("::surface.blur_px"))
    );
}

#[test]
fn rotten_club_main_menu_particle_emitters_use_component_metadata() {
    let graph = load_rotten_club_main_menu_graph();
    let particle = nodes_by_kind(&graph, AuthoringNodeKind::Component)
        .into_iter()
        .find(|node| {
            node.semantic.component_type.as_deref() == Some("ParticleEmitter2D")
                && node.semantic.owner_entity_name.as_deref() == Some("rain-10m")
        })
        .expect("rain-10m ParticleEmitter2D");
    let panel = build_property_panel_for_node(particle);
    let spawn_rate = property_by_suffix(&panel, "::spawn_rate");
    assert_eq!(spawn_rate.label, "Spawn Rate");
    assert!(matches!(
        spawn_rate.editor,
        AuthoringPropertyEditor::Slider { .. }
    ));
    assert_eq!(
        spawn_rate.binding,
        Some(AuthoringRuntimeBinding::ParticleEmitterProperty {
            entity_name: "rain-10m".to_owned(),
            field: "spawn_rate".to_owned(),
        })
    );
}

#[test]
fn rotten_club_main_menu_has_light_group_and_route_nodes() {
    let graph = load_rotten_club_main_menu_graph();
    let light_groups = nodes_by_kind(&graph, AuthoringNodeKind::LightGroup);
    let light_routes = nodes_by_kind(&graph, AuthoringNodeKind::LightRoute);
    assert!(
        light_groups
            .iter()
            .any(|node| node.semantic.light_group_id.as_deref() == Some("skyline")),
        "expected skyline light group"
    );
    assert!(
        light_routes.iter().any(|node| {
            node.semantic.light_route_receiver_layer.as_deref() == Some("weather.rain.near")
                || node.semantic.light_route_receiver_layer.as_deref() == Some("weather.rain.1m")
        }),
        "expected weather.rain.1m light route"
    );
}
