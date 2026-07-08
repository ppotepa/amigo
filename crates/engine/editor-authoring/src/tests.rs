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

fn test_particle_component_registry() -> amigo_scene::ComponentRegistry {
    let mut registry = amigo_scene::default_component_registry();
    registry
        .try_insert(test_layered_image_descriptor())
        .expect("test LayeredImage2D descriptor");
    registry
        .try_insert(test_particle_emitter_descriptor())
        .expect("test ParticleEmitter2D descriptor");
    registry
}

fn test_layered_image_descriptor() -> amigo_scene::ComponentTypeDescriptor {
    amigo_scene::ComponentTypeDescriptor {
        kind_id: "LayeredImage2D",
        type_name: "LayeredImage2D",
        label: "Layered Image 2D",
        domains: &[amigo_scene::ComponentDomain::Render2D],
        owner_scopes: &[amigo_scene::ComponentOwnerScope::Entity],
        default_yaml: None,
        metadata_traits: &[
            amigo_scene::MetadataTraitKind::Renderable2D,
            amigo_scene::MetadataTraitKind::RenderLayered2D,
            amigo_scene::MetadataTraitKind::HasAssetRefs,
            amigo_scene::MetadataTraitKind::RuntimeControllable,
            amigo_scene::MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[amigo_scene::ComponentAssetRefDescriptor {
            field_path: "asset",
            domain: amigo_scene::AssetDomain::LayeredImage,
            required: true,
            trait_kind: amigo_scene::MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            amigo_scene::EditorPropertyDescriptor {
                path: "asset",
                label: "Layered Image Asset",
                value_kind: amigo_scene::EditorPropertyValueKind::AssetRef,
                access: amigo_scene::EditorPropertyAccess::Editable,
                editor: amigo_scene::EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(amigo_scene::AssetDomain::LayeredImage),
                trait_kind: Some(amigo_scene::MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.primary",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: amigo_scene::EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            amigo_scene::EditorPropertyDescriptor {
                path: "size",
                label: "Size",
                value_kind: amigo_scene::EditorPropertyValueKind::Vec2,
                access: amigo_scene::EditorPropertyAccess::Editable,
                editor: amigo_scene::EditorPropertyEditorKind::Vec2,
                asset_domain: None,
                trait_kind: Some(amigo_scene::MetadataTraitKind::HasBounds2D),
                group: "bounds2.size",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: amigo_scene::EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            amigo_scene::EditorPropertyDescriptor {
                path: "base_opacity",
                label: "Base Opacity",
                value_kind: amigo_scene::EditorPropertyValueKind::Number,
                access: amigo_scene::EditorPropertyAccess::Editable,
                editor: amigo_scene::EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(amigo_scene::MetadataTraitKind::RuntimeControllable),
                group: "render2d.layers",
                patch_op: None,
                number_constraints: Some(amigo_scene::EDITOR_NUMBER_OPACITY),
                options: &[],
                visibility: amigo_scene::EditorPropertyVisibility::Primary,
                order: 0,
                tags: &["Live"],
                readonly_reason: None,
                binding_template: Some(
                    amigo_scene::EditorRuntimeBindingTemplate::LayeredImageBaseOpacity,
                ),
            },
        ],
        transform_policy: amigo_scene::TransformPolicy::UsesEntityTransform2,
        bounds_policy: amigo_scene::BoundsPolicy::ComponentBounds2D { field: "size" },
        editor_controls: &[
            amigo_scene::EditorControlKind::Transform2D,
            amigo_scene::EditorControlKind::Rect2D,
        ],
        patch_ops: &[amigo_scene::EditorPatchOpKind::SetTransform2],
    }
}

fn test_particle_emitter_descriptor() -> amigo_scene::ComponentTypeDescriptor {
    amigo_scene::ComponentTypeDescriptor {
        kind_id: "ParticleEmitter2D",
        type_name: "ParticleEmitter2D",
        label: "Particle Emitter 2D",
        domains: &[
            amigo_scene::ComponentDomain::Particles,
            amigo_scene::ComponentDomain::Render2D,
        ],
        owner_scopes: &[amigo_scene::ComponentOwnerScope::Entity],
        default_yaml: None,
        metadata_traits: &[
            amigo_scene::MetadataTraitKind::Renderable2D,
            amigo_scene::MetadataTraitKind::RuntimeControllable,
            amigo_scene::MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[
            amigo_scene::EditorPropertyDescriptor {
                path: "active",
                label: "Active",
                value_kind: amigo_scene::EditorPropertyValueKind::Bool,
                access: amigo_scene::EditorPropertyAccess::Editable,
                editor: amigo_scene::EditorPropertyEditorKind::Checkbox,
                asset_domain: None,
                trait_kind: Some(amigo_scene::MetadataTraitKind::RuntimeControllable),
                group: "render2d.content",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: amigo_scene::EditorPropertyVisibility::Primary,
                order: 0,
                tags: &["Live"],
                readonly_reason: None,
                binding_template: Some(
                    amigo_scene::EditorRuntimeBindingTemplate::ParticleEmitterField,
                ),
            },
            amigo_scene::EditorPropertyDescriptor {
                path: "spawn_rate",
                label: "Spawn Rate",
                value_kind: amigo_scene::EditorPropertyValueKind::Number,
                access: amigo_scene::EditorPropertyAccess::Editable,
                editor: amigo_scene::EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(amigo_scene::MetadataTraitKind::RuntimeControllable),
                group: "render2d.content",
                patch_op: None,
                number_constraints: Some(amigo_scene::EDITOR_NUMBER_PARTICLE_RATE),
                options: &[],
                visibility: amigo_scene::EditorPropertyVisibility::Primary,
                order: 0,
                tags: &["Live"],
                readonly_reason: None,
                binding_template: Some(
                    amigo_scene::EditorRuntimeBindingTemplate::ParticleEmitterField,
                ),
            },
        ],
        transform_policy: amigo_scene::TransformPolicy::UsesEntityTransform2,
        bounds_policy: amigo_scene::BoundsPolicy::SpawnArea2D {
            field: "spawn_area",
            size_field: "size",
            default_width: 64,
            default_height: 64,
        },
        editor_controls: &[
            amigo_scene::EditorControlKind::Transform2D,
            amigo_scene::EditorControlKind::Rect2D,
        ],
        patch_ops: &[amigo_scene::EditorPatchOpKind::SetTransform2],
    }
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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);

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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);

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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);

    assert_eq!(panel.title, "Component: Particle Emitter 2D");
    assert!(panel.groups.iter().any(|group| group.id == "metadata"));
    assert!(panel
        .groups
        .iter()
        .any(|group| group.id == "render2d.content"));
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
    assert!(panel
        .groups
        .iter()
        .flat_map(|group| group.properties.iter())
        .any(|property| property.label == "LayeredImage2D"));
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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);

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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);
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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);
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

    let registry = test_particle_component_registry();
    let panel = build_property_panel_for_node_with_registry(&node, &registry);

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
