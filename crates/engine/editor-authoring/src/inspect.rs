use amigo_editor_api::{
    AuthoringNumberConstraints, AuthoringOption, AuthoringProperty, AuthoringPropertyEditor,
    AuthoringPropertyGroup, AuthoringPropertyHints, AuthoringPropertyPanel, AuthoringPropertyValue,
    AuthoringRuntimeBinding,
};
use amigo_scene::{
    ComponentTypeDescriptor, EditorPropertyAccess, EditorPropertyDescriptor,
    EditorPropertyEditorKind, EditorPropertyValueKind, default_component_registry,
};
use serde_yaml::Value;

use crate::ids::child_pointer;
use crate::metadata_hints::{runtime_binding_hint, slider_hint_for_property};
use crate::{AuthoringNode, AuthoringNodeKind};

pub fn build_property_panel_for_node(node: &AuthoringNode) -> AuthoringPropertyPanel {
    match node.kind {
        AuthoringNodeKind::RenderLayer => render_layer_panel(node),
        AuthoringNodeKind::Component => component_panel(node),
        AuthoringNodeKind::PostFxItem => postfx_panel(node),
        AuthoringNodeKind::Entity => entity_panel(node),
        AuthoringNodeKind::PrefabRef => prefab_ref_panel(node),
        AuthoringNodeKind::PrefabOverrides => prefab_overrides_panel(node),
        AuthoringNodeKind::Use => use_ref_panel(node),
        AuthoringNodeKind::LightGroup => light_group_panel(node),
        AuthoringNodeKind::LightRoute => light_route_panel(node),
        AuthoringNodeKind::Scalar => scalar_leaf_panel(node),
        AuthoringNodeKind::Mapping => mapping_panel(node),
        AuthoringNodeKind::Sequence => sequence_panel(node),
        _ => generic_yaml_panel(node),
    }
}

fn render_layer_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let id = string_field(&node.value, "id").unwrap_or_else(|| "unknown".to_owned());
    AuthoringPropertyPanel {
        title: format!("Render Layer: {id}"),
        groups: vec![AuthoringPropertyGroup {
            id: "layer".to_owned(),
            title: "Layer".to_owned(),
            properties: vec![
                readonly_text(node, "id", id.clone()),
                text_row(
                    node,
                    "label",
                    string_field(&node.value, "label").unwrap_or_default(),
                ),
                slider_row(
                    node,
                    "order",
                    number_field(&node.value, "order").unwrap_or_default(),
                    -1000.0,
                    1000.0,
                    1.0,
                    Some(AuthoringRuntimeBinding::RenderLayerOrder {
                        layer_id: id.clone(),
                    }),
                ),
                toggle_row(
                    node,
                    "visible",
                    bool_field(&node.value, "visible").unwrap_or(true),
                    Some(AuthoringRuntimeBinding::RenderLayerVisible {
                        layer_id: id.clone(),
                    }),
                ),
                slider_row(
                    node,
                    "opacity",
                    number_field(&node.value, "opacity").unwrap_or(1.0),
                    0.0,
                    1.0,
                    0.01,
                    Some(AuthoringRuntimeBinding::RenderLayerOpacity { layer_id: id }),
                ),
            ],
        }],
    }
}

fn component_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let component_type = node
        .semantic
        .component_type
        .clone()
        .or_else(|| string_field(&node.value, "type"))
        .unwrap_or_else(|| "Component".to_owned());

    let registry = default_component_registry();
    let Some(descriptor) = registry.descriptor_by_type_name(&component_type) else {
        return generic_yaml_panel_with_title(node, format!("Component: {component_type}"));
    };

    component_panel_from_descriptor(node, descriptor)
}

fn component_panel_from_descriptor(
    node: &AuthoringNode,
    descriptor: &ComponentTypeDescriptor,
) -> AuthoringPropertyPanel {
    let mut groups = Vec::new();

    push_grouped(
        &mut groups,
        "metadata",
        "Metadata",
        readonly_text(node, "type", descriptor.type_name),
    );

    if let Some(owner) = node.semantic.owner_entity_name.clone() {
        push_grouped(
            &mut groups,
            "metadata",
            "Metadata",
            readonly_text(node, "entity", owner),
        );
    }

    for property_descriptor in descriptor.properties {
        let property = descriptor_property(node, property_descriptor);
        push_grouped(
            &mut groups,
            property_descriptor.group,
            property_descriptor.group,
            property,
        );
    }

    if descriptor.type_name == "LayeredImage2D" {
        append_layered_image_dynamic_properties(node, &mut groups);
    }

    AuthoringPropertyPanel {
        title: format!("Component: {}", descriptor.label),
        groups,
    }
}

fn descriptor_property(
    node: &AuthoringNode,
    descriptor: &EditorPropertyDescriptor,
) -> AuthoringProperty {
    let yaml_value = value_at_path(&node.value, descriptor.path);
    let null_value = Value::Null;

    AuthoringProperty {
        id: property_id(node, descriptor.path),
        label: descriptor.label.to_owned(),
        value: value_from_descriptor(descriptor.value_kind, yaml_value),
        editor: editor_from_descriptor(descriptor, yaml_value),
        hints: hints_from_descriptor(descriptor),
        read_only: matches!(descriptor.access, EditorPropertyAccess::ReadOnly),
        source_file: node.source_file.display().to_string(),
        yaml_pointer: property_yaml_pointer(node, descriptor.path),
        group: descriptor.group.to_owned(),
        trait_kind: descriptor.trait_kind.map(|kind| kind.id().to_owned()),
        binding: runtime_binding_hint(node, descriptor.path, yaml_value.unwrap_or(&null_value)),
    }
}

fn value_from_descriptor(
    kind: EditorPropertyValueKind,
    value: Option<&Value>,
) -> AuthoringPropertyValue {
    let Some(value) = value else {
        return AuthoringPropertyValue::Empty;
    };

    match kind {
        EditorPropertyValueKind::String => value
            .as_str()
            .map(|value| AuthoringPropertyValue::Text(value.to_owned()))
            .unwrap_or_else(|| AuthoringPropertyValue::Text(short_yaml(value))),
        EditorPropertyValueKind::AssetRef => value
            .as_str()
            .map(|value| AuthoringPropertyValue::AssetRef(value.to_owned()))
            .unwrap_or_else(|| AuthoringPropertyValue::AssetRef(short_yaml(value))),
        EditorPropertyValueKind::Enum => value
            .as_str()
            .map(|value| AuthoringPropertyValue::Enum(value.to_owned()))
            .unwrap_or_else(|| AuthoringPropertyValue::Enum(short_yaml(value))),
        EditorPropertyValueKind::Number => value
            .as_f64()
            .map(|value| AuthoringPropertyValue::Number(value as f32))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        EditorPropertyValueKind::Bool => value
            .as_bool()
            .map(AuthoringPropertyValue::Bool)
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        EditorPropertyValueKind::Vec2 => vec2_from_yaml(value)
            .map(|(x, y)| AuthoringPropertyValue::Vec2(x, y))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        EditorPropertyValueKind::Vec3 => vec3_from_yaml(value)
            .map(|(x, y, z)| AuthoringPropertyValue::Vec3(x, y, z))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        EditorPropertyValueKind::Color => AuthoringPropertyValue::Color(short_yaml(value)),
    }
}

fn editor_from_descriptor(
    descriptor: &EditorPropertyDescriptor,
    value: Option<&Value>,
) -> AuthoringPropertyEditor {
    if matches!(descriptor.access, EditorPropertyAccess::ReadOnly) {
        return AuthoringPropertyEditor::ReadOnly;
    }

    match descriptor.editor {
        EditorPropertyEditorKind::ReadOnly => AuthoringPropertyEditor::ReadOnly,
        EditorPropertyEditorKind::Text => AuthoringPropertyEditor::Text,
        EditorPropertyEditorKind::Checkbox => AuthoringPropertyEditor::Toggle,
        EditorPropertyEditorKind::Number => {
            let value = value.and_then(Value::as_f64).unwrap_or_default() as f32;
            let (fallback_min, fallback_max, fallback_step) =
                slider_hint_for_property(descriptor.path, value);
            let (min, max, step) = descriptor.number_constraints.map_or(
                (fallback_min, fallback_max, fallback_step),
                |constraints| {
                    (
                        constraints.min.unwrap_or(fallback_min),
                        constraints.max.unwrap_or(fallback_max),
                        constraints.step.unwrap_or(fallback_step),
                    )
                },
            );
            AuthoringPropertyEditor::Slider { min, max, step }
        }
        EditorPropertyEditorKind::AssetPicker => AuthoringPropertyEditor::AssetPicker {
            domain: descriptor
                .asset_domain
                .map(|domain| format!("{domain:?}"))
                .unwrap_or_else(|| "Raw".to_owned()),
        },
        EditorPropertyEditorKind::EnumSelect => AuthoringPropertyEditor::Enum {
            options: descriptor
                .options
                .iter()
                .map(|option| option.id.to_owned())
                .collect(),
        },
        EditorPropertyEditorKind::Color => AuthoringPropertyEditor::Color,
        EditorPropertyEditorKind::Vec2 => AuthoringPropertyEditor::Vec2,
        EditorPropertyEditorKind::Vec3 => AuthoringPropertyEditor::Vec3,
    }
}

fn hints_from_descriptor(descriptor: &EditorPropertyDescriptor) -> AuthoringPropertyHints {
    AuthoringPropertyHints {
        number: descriptor
            .number_constraints
            .map(|constraints| AuthoringNumberConstraints {
                min: constraints.min,
                max: constraints.max,
                step: constraints.step,
                clamp: constraints.clamp,
                unit: constraints.unit.map(str::to_owned),
                display_scale: constraints.display_scale,
            }),
        options: descriptor
            .options
            .iter()
            .map(|option| AuthoringOption {
                id: option.id.to_owned(),
                label: option.label.to_owned(),
            })
            .collect(),
    }
}

fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |current, segment| mapping_get(current, segment))
}

fn property_yaml_pointer(node: &AuthoringNode, path: &str) -> String {
    path.split('.')
        .fold(node.yaml_pointer.clone(), |pointer, segment| {
            child_pointer(&pointer, segment)
        })
}

fn push_grouped(
    groups: &mut Vec<AuthoringPropertyGroup>,
    id: &str,
    title: &str,
    mut property: AuthoringProperty,
) {
    property.group = id.to_owned();
    if let Some(group) = groups.iter_mut().find(|group| group.id == id) {
        group.properties.push(property);
        return;
    }
    groups.push(AuthoringPropertyGroup {
        id: id.to_owned(),
        title: title.to_owned(),
        properties: vec![property],
    });
}

fn append_layered_image_dynamic_properties(
    node: &AuthoringNode,
    groups: &mut Vec<AuthoringPropertyGroup>,
) {
    let entity_name = node
        .semantic
        .owner_entity_name
        .clone()
        .unwrap_or_else(|| "unknown-entity".to_owned());

    let Some(overrides) = mapping_get(&node.value, "layer_overrides").and_then(Value::as_sequence)
    else {
        return;
    };

    for item in overrides {
        let Some(layer_id) = string_field(item, "id") else {
            continue;
        };

        let opacity = number_field(item, "opacity").unwrap_or(1.0);
        let (min, max, step) = slider_hint_for_property("layer_overrides.opacity", opacity);

        let opacity_path = format!("layer_overrides.{layer_id}.opacity");
        let mut opacity_property = slider_row(
            node,
            &opacity_path,
            opacity,
            min,
            max,
            step,
            Some(AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                entity_name: entity_name.clone(),
                layer_id: layer_id.clone(),
            }),
        );
        opacity_property.label = format!("{layer_id} opacity");

        push_grouped(
            groups,
            "render2d.layers.dynamic",
            "Layer Overrides",
            opacity_property,
        );

        let enabled_path = format!("layer_overrides.{layer_id}.enabled");
        let mut enabled_property = toggle_row(
            node,
            &enabled_path,
            bool_field(item, "enabled").unwrap_or(true),
            Some(AuthoringRuntimeBinding::LayeredImageLayerEnabled {
                entity_name: entity_name.clone(),
                layer_id: layer_id.clone(),
            }),
        );
        enabled_property.label = format!("{layer_id} enabled");

        push_grouped(
            groups,
            "render2d.layers.dynamic",
            "Layer Overrides",
            enabled_property,
        );
    }
}

fn postfx_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let effect_type = string_field(&node.value, "type").unwrap_or_else(|| "effect".to_owned());
    let effect_id = string_field(&node.value, "id").unwrap_or_else(|| effect_type.clone());
    let mut properties = vec![
        readonly_text(node, "type", effect_type),
        readonly_text(node, "id", effect_id.clone()),
    ];
    collect_postfx_properties(node, &effect_id, "", &node.value, &mut properties);
    AuthoringPropertyPanel {
        title: format!("Post FX: {effect_id}"),
        groups: vec![AuthoringPropertyGroup {
            id: "effect".to_owned(),
            title: "Effect".to_owned(),
            properties,
        }],
    }
}

fn collect_postfx_properties(
    node: &AuthoringNode,
    effect_id: &str,
    path: &str,
    value: &Value,
    out: &mut Vec<AuthoringProperty>,
) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map {
                let Some(key) = k.as_str() else { continue };
                if key == "type" || key == "id" {
                    continue;
                }
                let next = if path.is_empty() {
                    key.to_owned()
                } else {
                    format!("{path}.{key}")
                };
                collect_postfx_properties(node, effect_id, &next, v, out);
            }
        }
        Value::Bool(v) => out.push(toggle_row(
            node,
            path,
            *v,
            Some(AuthoringRuntimeBinding::PostFxMock {
                effect_id: effect_id.to_owned(),
                field: path.to_owned(),
            }),
        )),
        Value::Number(n) => {
            let value = n.as_f64().unwrap_or_default() as f32;
            let (min, max, step) = slider_hint_for_property(path, value);
            out.push(slider_row(
                node,
                path,
                value,
                min,
                max,
                step,
                Some(AuthoringRuntimeBinding::PostFxMock {
                    effect_id: effect_id.to_owned(),
                    field: path.to_owned(),
                }),
            ));
        }
        Value::String(s) => out.push(readonly_text(node, path, s)),
        Value::Sequence(items) => {
            out.push(readonly_text(node, path, format!("{} items", items.len())))
        }
        Value::Null => out.push(readonly_text(node, path, "null")),
        Value::Tagged(tagged) => {
            collect_postfx_properties(node, effect_id, path, &tagged.value, out)
        }
    }
}

fn entity_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    generic_yaml_panel_with_title(
        node,
        format!(
            "Entity: {}",
            string_field(&node.value, "name")
                .or_else(|| string_field(&node.value, "id"))
                .unwrap_or_else(|| "unknown".to_owned())
        ),
    )
}

fn scalar_leaf_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: format!("Leaf: {}", node.label),
        groups: vec![AuthoringPropertyGroup {
            id: "leaf".to_owned(),
            title: "Leaf".to_owned(),
            properties: vec![
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "kind", format!("{:?}", node.kind)),
                readonly_text(node, "value", short_yaml(&node.value)),
                readonly_text(node, "value_type", yaml_type_name(&node.value)),
            ],
        }],
    }
}

fn mapping_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let count = node
        .value
        .as_mapping()
        .map(|mapping| mapping.len())
        .unwrap_or(0);
    generic_summary_panel(node, "Mapping", count, "fields")
}

fn sequence_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let count = node
        .value
        .as_sequence()
        .map(|sequence| sequence.len())
        .unwrap_or(0);
    generic_summary_panel(node, "Sequence", count, "items")
}

fn generic_summary_panel(
    node: &AuthoringNode,
    title: &str,
    count: usize,
    count_label: &str,
) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: format!("{title}: {}", node.label),
        groups: vec![AuthoringPropertyGroup {
            id: "summary".to_owned(),
            title: "Summary".to_owned(),
            properties: vec![
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "kind", format!("{:?}", node.kind)),
                readonly_text(node, count_label, count.to_string()),
                readonly_text(node, "preview", node.value_preview.clone()),
            ],
        }],
    }
}

fn prefab_ref_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let prefab = match &node.value {
        Value::String(value) => value.clone(),
        _ => short_yaml(&node.value),
    };
    AuthoringPropertyPanel {
        title: "Prefab Reference".to_owned(),
        groups: vec![AuthoringPropertyGroup {
            id: "prefab".to_owned(),
            title: "Prefab".to_owned(),
            properties: vec![
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "prefab", prefab),
                readonly_text(node, "editable", "false"),
            ],
        }],
    }
}

fn prefab_overrides_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let mut properties = vec![
        readonly_text(node, "source", node.source_file.display().to_string()),
        readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
        readonly_text(node, "editable", "false"),
    ];
    match node.value.as_mapping() {
        Some(mapping) => {
            properties.push(readonly_text(
                node,
                "override_count",
                mapping.len().to_string(),
            ));
            for (key, value) in mapping {
                let Some(key) = key.as_str() else { continue };
                properties.push(readonly_text(node, key, short_yaml(value)));
            }
        }
        None => properties.push(readonly_text(node, "value", short_yaml(&node.value))),
    }
    AuthoringPropertyPanel {
        title: "Prefab Overrides".to_owned(),
        groups: vec![AuthoringPropertyGroup {
            id: "prefab.overrides".to_owned(),
            title: "Prefab Overrides".to_owned(),
            properties,
        }],
    }
}

fn use_ref_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: format!("Use Ref: {}", node.label),
        groups: vec![AuthoringPropertyGroup {
            id: "use".to_owned(),
            title: "Use Reference".to_owned(),
            properties: vec![
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "origin", format!("{:?}", node.origin)),
                readonly_text(node, "resolved", "true"),
                readonly_text(node, "children", node.children.len().to_string()),
            ],
        }],
    }
}

fn light_group_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let subject = node
        .semantic
        .light_group_id
        .clone()
        .or_else(|| string_field(&node.value, "id"))
        .or_else(|| string_field(&node.value, "name"))
        .unwrap_or_else(|| "unknown-light-group".to_owned());
    mock_yaml_panel(
        node,
        format!("Light Group: {subject}"),
        "light_group",
        subject,
    )
}

fn light_route_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let subject = node
        .semantic
        .light_route_receiver_layer
        .clone()
        .or_else(|| string_field(&node.value, "receiver_layer"))
        .or_else(|| string_field(&node.value, "layer"))
        .unwrap_or_else(|| "unknown-light-route".to_owned());
    mock_yaml_panel(
        node,
        format!("Light Route: {subject}"),
        "light_route",
        subject,
    )
}

fn mock_yaml_panel(
    node: &AuthoringNode,
    title: String,
    namespace: &str,
    subject: String,
) -> AuthoringPropertyPanel {
    let mut properties = vec![
        readonly_text(node, "source", node.source_file.display().to_string()),
        readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
        readonly_text(node, "kind", format!("{:?}", node.kind)),
    ];
    if let Some(mapping) = node.value.as_mapping() {
        for (key, value) in mapping {
            let Some(key) = key.as_str() else { continue };
            properties.push(mock_yaml_row(node, key, value, namespace, &subject));
        }
    } else {
        properties.push(readonly_text(node, "value", short_yaml(&node.value)));
    }
    AuthoringPropertyPanel {
        title,
        groups: vec![AuthoringPropertyGroup {
            id: namespace.to_owned(),
            title: namespace.replace('_', " "),
            properties,
        }],
    }
}

fn mock_yaml_row(
    node: &AuthoringNode,
    label: &str,
    value: &Value,
    namespace: &str,
    subject: &str,
) -> AuthoringProperty {
    match value {
        Value::Bool(value) => toggle_row(
            node,
            label,
            *value,
            Some(AuthoringRuntimeBinding::Mock {
                namespace: namespace.to_owned(),
                subject: subject.to_owned(),
                field: label.to_owned(),
            }),
        ),
        Value::Number(number) => {
            let value = number.as_f64().unwrap_or_default() as f32;
            let (min, max, step) = slider_hint_for_property(label, value);
            slider_row(
                node,
                label,
                value,
                min,
                max,
                step,
                Some(AuthoringRuntimeBinding::Mock {
                    namespace: namespace.to_owned(),
                    subject: subject.to_owned(),
                    field: label.to_owned(),
                }),
            )
        }
        Value::String(value) => readonly_text(node, label, value),
        Value::Sequence(items) => readonly_text(node, label, format!("{} items", items.len())),
        Value::Mapping(mapping) => readonly_text(node, label, format!("{} fields", mapping.len())),
        Value::Null => readonly_text(node, label, "null"),
        Value::Tagged(tagged) => mock_yaml_row(node, label, &tagged.value, namespace, subject),
    }
}

fn generic_yaml_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    generic_yaml_panel_with_title(node, node.label.clone())
}

fn generic_yaml_panel_with_title(node: &AuthoringNode, title: String) -> AuthoringPropertyPanel {
    let mut properties = vec![
        readonly_text(node, "source", node.source_file.display().to_string()),
        readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
        readonly_text(node, "kind", format!("{:?}", node.kind)),
    ];
    if let Some(mapping) = node.value.as_mapping() {
        for (k, v) in mapping {
            let Some(key) = k.as_str() else { continue };
            properties.push(generic_row(node, key, v));
        }
    }
    AuthoringPropertyPanel {
        title,
        groups: vec![AuthoringPropertyGroup {
            id: "yaml".to_owned(),
            title: "YAML".to_owned(),
            properties,
        }],
    }
}

fn property_id(node: &AuthoringNode, path: &str) -> String {
    format!("{}::{}", node.id, path)
}

fn generic_row(node: &AuthoringNode, label: &str, value: &Value) -> AuthoringProperty {
    match value {
        Value::Bool(v) => toggle_row(node, label, *v, None),
        Value::Number(n) => number_row(node, label, n.as_f64().unwrap_or_default() as f32),
        Value::String(v) => text_row(node, label, v.clone()),
        _ => readonly_text(node, label, short_yaml(value)),
    }
}

fn readonly_text(node: &AuthoringNode, label: &str, value: impl Into<String>) -> AuthoringProperty {
    AuthoringProperty {
        id: property_id(node, label),
        label: label.to_owned(),
        value: AuthoringPropertyValue::Text(value.into()),
        editor: AuthoringPropertyEditor::ReadOnly,
        hints: AuthoringPropertyHints::default(),
        read_only: true,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        group: "default".to_owned(),
        trait_kind: None,
        binding: None,
    }
}

fn text_row(node: &AuthoringNode, label: &str, value: impl Into<String>) -> AuthoringProperty {
    AuthoringProperty {
        id: property_id(node, label),
        label: label.to_owned(),
        value: AuthoringPropertyValue::Text(value.into()),
        editor: AuthoringPropertyEditor::Text,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        group: "default".to_owned(),
        trait_kind: None,
        binding: None,
    }
}

fn number_row(node: &AuthoringNode, label: &str, value: f32) -> AuthoringProperty {
    AuthoringProperty {
        id: property_id(node, label),
        label: label.to_owned(),
        value: AuthoringPropertyValue::Number(value),
        editor: AuthoringPropertyEditor::Number,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        group: "default".to_owned(),
        trait_kind: None,
        binding: None,
    }
}

fn slider_row(
    node: &AuthoringNode,
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    binding: Option<AuthoringRuntimeBinding>,
) -> AuthoringProperty {
    let binding = binding.or_else(|| {
        let yaml = value_to_yaml_number(value);
        runtime_binding_hint(node, label, &yaml)
    });
    AuthoringProperty {
        id: property_id(node, label),
        label: label.to_owned(),
        value: AuthoringPropertyValue::Number(value),
        editor: AuthoringPropertyEditor::Slider { min, max, step },
        hints: AuthoringPropertyHints {
            number: Some(AuthoringNumberConstraints {
                min: Some(min),
                max: Some(max),
                step: Some(step),
                clamp: true,
                unit: None,
                display_scale: 1.0,
            }),
            options: Vec::new(),
        },
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        group: "default".to_owned(),
        trait_kind: None,
        binding,
    }
}

fn toggle_row(
    node: &AuthoringNode,
    label: &str,
    value: bool,
    binding: Option<AuthoringRuntimeBinding>,
) -> AuthoringProperty {
    AuthoringProperty {
        id: property_id(node, label),
        label: label.to_owned(),
        value: AuthoringPropertyValue::Bool(value),
        editor: AuthoringPropertyEditor::Toggle,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        group: "default".to_owned(),
        trait_kind: None,
        binding,
    }
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}
fn string_field(value: &Value, key: &str) -> Option<String> {
    mapping_get(value, key)?.as_str().map(str::to_owned)
}
fn number_field(value: &Value, key: &str) -> Option<f32> {
    mapping_get(value, key)?.as_f64().map(|v| v as f32)
}
fn bool_field(value: &Value, key: &str) -> Option<bool> {
    mapping_get(value, key)?.as_bool()
}
fn short_yaml(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        Value::Sequence(items) => format!("{} items", items.len()),
        Value::Mapping(mapping) => format!("{} fields", mapping.len()),
        Value::Tagged(tagged) => short_yaml(&tagged.value),
    }
}

fn yaml_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "sequence",
        Value::Mapping(_) => "mapping",
        Value::Tagged(_) => "tagged",
    }
}

fn value_to_yaml_number(value: f32) -> Value {
    match serde_yaml::to_value(value) {
        Ok(v) => v,
        Err(_) => Value::Null,
    }
}

fn vec2_from_yaml(value: &Value) -> Option<(f32, f32)> {
    Some((
        mapping_get(value, "x")?.as_f64()? as f32,
        mapping_get(value, "y")?.as_f64()? as f32,
    ))
}

fn vec3_from_yaml(value: &Value) -> Option<(f32, f32, f32)> {
    Some((
        mapping_get(value, "x")?.as_f64()? as f32,
        mapping_get(value, "y")?.as_f64()? as f32,
        mapping_get(value, "z")?.as_f64()? as f32,
    ))
}
