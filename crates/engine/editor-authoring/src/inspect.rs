use amigo_editor_api::{
    AuthoringNumberConstraints, AuthoringOption, AuthoringProperty, AuthoringPropertyApplyMode,
    AuthoringPropertyDisplay, AuthoringPropertyEditor, AuthoringPropertyGroup,
    AuthoringPropertyHints, AuthoringPropertyPanel, AuthoringPropertyValue,
    AuthoringPropertyVisibility, AuthoringRuntimeBinding,
};
use amigo_scene::{
    ComponentTypeDescriptor, EditorPropertyAccess, EditorPropertyDescriptor,
    EditorPropertyEditorKind, EditorPropertyValueKind,
    EditorPropertyVisibility as ScenePropertyVisibility, default_component_registry,
};
use serde_yaml::Value;

use crate::bindings::resolve_property_binding;
use crate::ids::child_pointer;
use crate::image_parts::collect_image_part_properties;
use crate::node_descriptors::{RENDER_LAYER_PROPERTIES, property_from_node_descriptor};
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
        AuthoringNodeKind::Scalar | AuthoringNodeKind::Mapping | AuthoringNodeKind::Sequence => {
            raw_debug_only_panel(node)
        }
        _ => semantic_status_panel(node, node.label.clone(), "No descriptor-backed properties"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorViewMode {
    Primary,
    Advanced,
    RawDebug,
}

pub fn filter_property_panel_for_view(
    mut panel: AuthoringPropertyPanel,
    mode: InspectorViewMode,
) -> AuthoringPropertyPanel {
    for group in &mut panel.groups {
        group
            .properties
            .retain(|row| property_visible_for_view(row, mode));
    }
    panel.groups.retain(|group| !group.properties.is_empty());
    panel
}

fn property_visible_for_view(row: &AuthoringProperty, mode: InspectorViewMode) -> bool {
    match mode {
        InspectorViewMode::Primary => {
            matches!(row.display.visibility, AuthoringPropertyVisibility::Primary)
        }
        InspectorViewMode::Advanced => {
            !matches!(row.display.visibility, AuthoringPropertyVisibility::Hidden)
        }
        InspectorViewMode::RawDebug => true,
    }
}

fn display_for_binding(
    binding: &Option<AuthoringRuntimeBinding>,
    read_only: bool,
    visibility: AuthoringPropertyVisibility,
    mut tags: Vec<String>,
) -> AuthoringPropertyDisplay {
    let apply_mode = if read_only {
        AuthoringPropertyApplyMode::ReadOnly
    } else {
        match binding {
            Some(AuthoringRuntimeBinding::Mock { .. })
            | Some(AuthoringRuntimeBinding::PostFxMock { .. }) => AuthoringPropertyApplyMode::Mock,
            Some(_) => AuthoringPropertyApplyMode::Live,
            None => AuthoringPropertyApplyMode::Unsupported,
        }
    };

    match apply_mode {
        AuthoringPropertyApplyMode::Live => tags.push("Live".to_owned()),
        AuthoringPropertyApplyMode::Mock => tags.push("Mock".to_owned()),
        AuthoringPropertyApplyMode::ReadOnly => tags.push("Readonly".to_owned()),
        AuthoringPropertyApplyMode::Unsupported => tags.push("Unsupported".to_owned()),
    }

    AuthoringPropertyDisplay {
        icon: None,
        tags,
        visibility,
        apply_mode,
        order: 0,
    }
}

fn group_label(group: &str) -> &'static str {
    if group.starts_with("render2d") {
        "Render"
    } else if group.starts_with("transform") {
        "Transform"
    } else if group.starts_with("particles") {
        "Particles"
    } else if group.starts_with("asset") {
        "Assets"
    } else if group.starts_with("metadata") {
        "Metadata"
    } else {
        "General"
    }
}

fn render_layer_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let id = string_field(&node.value, "id").unwrap_or_else(|| "unknown".to_owned());
    let properties = RENDER_LAYER_PROPERTIES
        .iter()
        .map(|descriptor| property_from_node_descriptor(node, descriptor, Some(id.as_str())))
        .collect();

    AuthoringPropertyPanel {
        title: format!("Draw Layer: {id}"),
        groups: vec![AuthoringPropertyGroup {
            id: "render".to_owned(),
            title: "Render".to_owned(),
            properties,
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
        return semantic_status_panel(
            node,
            format!("Component: {component_type}"),
            "No component descriptor yet",
        );
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
            group_label(property_descriptor.group),
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
    let binding = resolve_property_binding(node, descriptor, yaml_value);
    let read_only = matches!(descriptor.access, EditorPropertyAccess::ReadOnly);
    let trait_kind = descriptor.trait_kind.map(|kind| kind.id().to_owned());
    let visibility = authoring_visibility(descriptor.visibility);
    let mut tags = descriptor
        .tags
        .iter()
        .map(|tag| (*tag).to_owned())
        .collect::<Vec<_>>();
    if !read_only && binding.is_none() {
        tags.push("NoBinding".to_owned());
    }

    AuthoringProperty {
        id: property_id(node, descriptor.path),
        label: descriptor.label.to_owned(),
        value: value_from_descriptor(descriptor.value_kind, yaml_value),
        editor: editor_from_descriptor(descriptor),
        hints: hints_from_descriptor(descriptor),
        read_only,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: property_yaml_pointer(node, descriptor.path),
        group: descriptor.group.to_owned(),
        trait_kind,
        binding: binding.clone(),
        display: display_for_binding(&binding, read_only, visibility, tags),
    }
}

fn authoring_visibility(visibility: ScenePropertyVisibility) -> AuthoringPropertyVisibility {
    match visibility {
        ScenePropertyVisibility::Primary => AuthoringPropertyVisibility::Primary,
        ScenePropertyVisibility::Advanced => AuthoringPropertyVisibility::Advanced,
        ScenePropertyVisibility::Debug => AuthoringPropertyVisibility::Debug,
        ScenePropertyVisibility::Hidden => AuthoringPropertyVisibility::Hidden,
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

fn editor_from_descriptor(descriptor: &EditorPropertyDescriptor) -> AuthoringPropertyEditor {
    if matches!(descriptor.access, EditorPropertyAccess::ReadOnly) {
        return AuthoringPropertyEditor::ReadOnly;
    }

    match descriptor.editor {
        EditorPropertyEditorKind::ReadOnly => AuthoringPropertyEditor::ReadOnly,
        EditorPropertyEditorKind::Text => AuthoringPropertyEditor::Text,
        EditorPropertyEditorKind::Checkbox => AuthoringPropertyEditor::Toggle,
        EditorPropertyEditorKind::Number => {
            if let Some(constraints) = descriptor.number_constraints {
                AuthoringPropertyEditor::Slider {
                    min: constraints.min.unwrap_or(0.0),
                    max: constraints.max.unwrap_or(1.0),
                    step: constraints.step.unwrap_or(0.01),
                }
            } else {
                AuthoringPropertyEditor::Number
            }
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
    for property in collect_image_part_properties(node) {
        push_grouped(
            groups,
            "render2d.image_parts.dynamic",
            "Image Parts",
            property,
        );
    }
}

fn postfx_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let effect_type = string_field(&node.value, "type").unwrap_or_else(|| "effect".to_owned());
    let effect_id = string_field(&node.value, "id").unwrap_or_else(|| effect_type.clone());
    semantic_status_panel(
        node,
        format!("Post FX: {effect_id}"),
        format!("{effect_type}: No live runtime binding yet"),
    )
}

fn entity_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let name = string_field(&node.value, "name")
        .or_else(|| string_field(&node.value, "id"))
        .unwrap_or_else(|| "unknown".to_owned());
    let mut components = Vec::new();
    collect_component_nodes(node, &mut components);
    let component_count = components.len();
    let mut groups = vec![
        AuthoringPropertyGroup {
            id: "summary".to_owned(),
            title: "Summary".to_owned(),
            properties: vec![
                status_text_primary(node, "name", name.as_str()),
                status_text_primary(node, "components", component_count.to_string()),
                readonly_text(node, "source", node.source_file.display().to_string()),
            ],
        },
        AuthoringPropertyGroup {
            id: "components".to_owned(),
            title: "Components".to_owned(),
            properties: entity_component_rows(&components, node),
        },
    ];
    AuthoringPropertyPanel {
        title: format!("Entity: {name}"),
        groups,
    }
}

fn collect_component_nodes<'a>(node: &'a AuthoringNode, out: &mut Vec<&'a AuthoringNode>) {
    for child in &node.children {
        if matches!(child.kind, AuthoringNodeKind::Component) {
            out.push(child);
        }
        collect_component_nodes(child, out);
    }
}

fn entity_component_rows(components: &[&AuthoringNode], fallback_node: &AuthoringNode) -> Vec<AuthoringProperty> {
    let mut rows = Vec::new();
    for child in components {
        let component_type = child
            .semantic
            .component_type
            .clone()
            .or_else(|| string_field(&child.value, "type"))
            .unwrap_or_else(|| "Component".to_owned());
        rows.push(readonly_text(
            child,
            component_type.as_str(),
            "descriptor-backed",
        ));
    }
    if rows.is_empty() {
        rows.push(readonly_text(fallback_node, "components", "none"));
    }
    rows
}

fn raw_debug_only_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    semantic_status_panel(
        node,
        format!("Raw Debug: {}", node.label),
        "Raw YAML is available only in Raw Debug",
    )
}

fn semantic_status_panel(
    node: &AuthoringNode,
    title: impl Into<String>,
    status: impl Into<String>,
) -> AuthoringPropertyPanel {
    AuthoringPropertyPanel {
        title: title.into(),
        groups: vec![AuthoringPropertyGroup {
            id: "status".to_owned(),
            title: "Status".to_owned(),
            properties: vec![
                status_text_primary(node, "status", status),
                readonly_text(node, "source", node.source_file.display().to_string()),
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
                status_text_primary(node, "prefab", prefab),
                readonly_text(node, "source", node.source_file.display().to_string()),
                readonly_text(node, "yaml_pointer", node.yaml_pointer.clone()),
                readonly_text(node, "editable", "false"),
            ],
        }],
    }
}

fn prefab_overrides_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let mut properties = vec![
        status_text_primary(node, "status", "Readonly"),
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
                status_text_primary(node, "status", "Readonly"),
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
    semantic_status_panel(
        node,
        format!("Light Group: {subject}"),
        "No live runtime binding yet",
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
    semantic_status_panel(
        node,
        format!("Light Route: {subject}"),
        "No live runtime binding yet",
    )
}

fn property_id(node: &AuthoringNode, path: &str) -> String {
    format!("{}::{}", node.id, path)
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
        group: "metadata".to_owned(),
        trait_kind: None,
        binding: None,
        display: display_for_binding(
            &None,
            true,
            AuthoringPropertyVisibility::Advanced,
            vec!["Metadata".to_owned()],
        ),
    }
}

fn status_text_primary(
    node: &AuthoringNode,
    label: &str,
    value: impl Into<String>,
) -> AuthoringProperty {
    let mut property = readonly_text(node, label, value);
    property.display.visibility = AuthoringPropertyVisibility::Primary;
    property.display.tags = vec!["Readonly".to_owned()];
    property
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    mapping_get(value, key)?.as_str().map(str::to_owned)
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
