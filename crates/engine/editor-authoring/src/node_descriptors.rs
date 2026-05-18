use amigo_editor_api::{
    AuthoringNumberConstraints, AuthoringProperty, AuthoringPropertyApplyMode,
    AuthoringPropertyDisplay, AuthoringPropertyEditor, AuthoringPropertyHints,
    AuthoringPropertyValue, AuthoringPropertyVisibility,
};
use amigo_scene::{
    EDITOR_NUMBER_OPACITY, EDITOR_NUMBER_ORDER, EDITOR_NUMBER_Z_DEPTH, EditorNumberConstraints,
    EditorPropertyAccess, EditorPropertyDescriptor, EditorPropertyEditorKind,
    EditorPropertyValueKind, EditorPropertyVisibility as ScenePropertyVisibility,
    EditorRuntimeBindingTemplate,
};
use serde_yaml::Value;

use crate::AuthoringNode;
use crate::bindings::resolve_property_binding;
use crate::ids::child_pointer;

pub const RENDER_LAYER_PROPERTIES: &[EditorPropertyDescriptor] = &[
    EditorPropertyDescriptor {
        path: "id",
        label: "Id",
        value_kind: EditorPropertyValueKind::String,
        access: EditorPropertyAccess::ReadOnly,
        editor: EditorPropertyEditorKind::ReadOnly,
        asset_domain: None,
        trait_kind: None,
        group: "Metadata",
        patch_op: None,
        number_constraints: None,
        options: &[],
        visibility: ScenePropertyVisibility::Advanced,
        order: 0,
        tags: &["DrawLayer", "Readonly"],
        readonly_reason: Some("Draw Layer id is runtime identity"),
        binding_template: None,
    },
    EditorPropertyDescriptor {
        path: "label",
        label: "Label",
        value_kind: EditorPropertyValueKind::String,
        access: EditorPropertyAccess::ReadOnly,
        editor: EditorPropertyEditorKind::ReadOnly,
        asset_domain: None,
        trait_kind: None,
        group: "Metadata",
        patch_op: None,
        number_constraints: None,
        options: &[],
        visibility: ScenePropertyVisibility::Advanced,
        order: 1,
        tags: &["DrawLayer", "Readonly"],
        readonly_reason: Some("No live runtime binding yet"),
        binding_template: None,
    },
    EditorPropertyDescriptor {
        path: "order",
        label: "Order",
        value_kind: EditorPropertyValueKind::Number,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Number,
        asset_domain: None,
        trait_kind: None,
        group: "Render",
        patch_op: None,
        number_constraints: Some(EDITOR_NUMBER_ORDER),
        options: &[],
        visibility: ScenePropertyVisibility::Primary,
        order: 10,
        tags: &["DrawLayer", "Live"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "visible",
        label: "Visible",
        value_kind: EditorPropertyValueKind::Bool,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Checkbox,
        asset_domain: None,
        trait_kind: None,
        group: "Render",
        patch_op: None,
        number_constraints: None,
        options: &[],
        visibility: ScenePropertyVisibility::Primary,
        order: 20,
        tags: &["DrawLayer", "Live"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "opacity",
        label: "Opacity",
        value_kind: EditorPropertyValueKind::Number,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Number,
        asset_domain: None,
        trait_kind: None,
        group: "Render",
        patch_op: None,
        number_constraints: Some(EDITOR_NUMBER_OPACITY),
        options: &[],
        visibility: ScenePropertyVisibility::Primary,
        order: 30,
        tags: &["DrawLayer", "Live"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "depth.mode",
        label: "Depth Mode",
        value_kind: EditorPropertyValueKind::String,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::EnumSelect,
        asset_domain: None,
        trait_kind: None,
        group: "Depth",
        patch_op: None,
        number_constraints: None,
        options: &[
            amigo_scene::EditorPropertyOption {
                id: "depth_map",
                label: "Depth Map",
            },
            amigo_scene::EditorPropertyOption {
                id: "distance",
                label: "Distance",
            },
            amigo_scene::EditorPropertyOption {
                id: "z_depth",
                label: "Z Depth",
            },
            amigo_scene::EditorPropertyOption {
                id: "infinity",
                label: "Infinity",
            },
            amigo_scene::EditorPropertyOption {
                id: "overlay",
                label: "Overlay",
            },
        ],
        visibility: ScenePropertyVisibility::Primary,
        order: 40,
        tags: &["DrawLayer", "Live", "Depth"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "depth.distance_m",
        label: "Distance (m)",
        value_kind: EditorPropertyValueKind::Number,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Number,
        asset_domain: None,
        trait_kind: None,
        group: "Depth",
        patch_op: None,
        number_constraints: Some(EditorNumberConstraints {
            min: Some(0.0),
            max: Some(10000.0),
            step: Some(0.1),
            clamp: true,
            unit: Some("m"),
            display_scale: 1.0,
        }),
        options: &[],
        visibility: ScenePropertyVisibility::Primary,
        order: 45,
        tags: &["DrawLayer", "Live", "Depth", "Authoring"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "depth.z_depth",
        label: "Z Depth Override / Computed",
        value_kind: EditorPropertyValueKind::Number,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Number,
        asset_domain: None,
        trait_kind: None,
        group: "Depth",
        patch_op: None,
        number_constraints: Some(EDITOR_NUMBER_Z_DEPTH),
        options: &[],
        visibility: ScenePropertyVisibility::Advanced,
        order: 50,
        tags: &["DrawLayer", "Live", "Depth"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
    EditorPropertyDescriptor {
        path: "depth.blur_scale",
        label: "Depth Blur Scale",
        value_kind: EditorPropertyValueKind::Number,
        access: EditorPropertyAccess::Editable,
        editor: EditorPropertyEditorKind::Number,
        asset_domain: None,
        trait_kind: None,
        group: "Depth",
        patch_op: None,
        number_constraints: Some(EditorNumberConstraints {
            min: Some(0.0),
            max: Some(4.0),
            step: Some(0.01),
            clamp: true,
            unit: None,
            display_scale: 1.0,
        }),
        options: &[],
        visibility: ScenePropertyVisibility::Advanced,
        order: 60,
        tags: &["DrawLayer", "Live", "Depth"],
        readonly_reason: None,
        binding_template: Some(EditorRuntimeBindingTemplate::RenderLayerField),
    },
];

pub fn property_from_node_descriptor(
    node: &AuthoringNode,
    descriptor: &EditorPropertyDescriptor,
    _layer_id: Option<&str>,
) -> AuthoringProperty {
    let yaml_value = value_at_path(&node.value, descriptor.path);
    let binding = resolve_property_binding(node, descriptor, yaml_value);
    let read_only = matches!(descriptor.access, EditorPropertyAccess::ReadOnly);

    AuthoringProperty {
        id: format!("{}::{}", node.id, descriptor.path),
        label: descriptor.label.to_owned(),
        value: value_from_descriptor(descriptor.value_kind, yaml_value),
        editor: editor_from_descriptor(descriptor),
        hints: hints_from_descriptor(descriptor),
        read_only,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: descriptor
            .path
            .split('.')
            .fold(node.yaml_pointer.clone(), |pointer, segment| {
                child_pointer(&pointer, segment)
            }),
        group: descriptor.group.to_owned(),
        trait_kind: None,
        binding: binding.clone(),
        display: display_from_descriptor(descriptor, read_only, binding.is_some()),
    }
}

fn display_from_descriptor(
    descriptor: &EditorPropertyDescriptor,
    read_only: bool,
    has_binding: bool,
) -> AuthoringPropertyDisplay {
    let apply_mode = if read_only {
        AuthoringPropertyApplyMode::ReadOnly
    } else if has_binding {
        AuthoringPropertyApplyMode::Live
    } else {
        AuthoringPropertyApplyMode::Unsupported
    };

    AuthoringPropertyDisplay {
        icon: None,
        tags: descriptor
            .tags
            .iter()
            .map(|tag| (*tag).to_owned())
            .collect(),
        visibility: authoring_visibility(descriptor.visibility),
        apply_mode,
        order: descriptor.order,
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

fn editor_from_descriptor(descriptor: &EditorPropertyDescriptor) -> AuthoringPropertyEditor {
    match descriptor.editor {
        EditorPropertyEditorKind::ReadOnly => AuthoringPropertyEditor::ReadOnly,
        EditorPropertyEditorKind::Text => AuthoringPropertyEditor::Text,
        EditorPropertyEditorKind::Number => match descriptor.number_constraints {
            Some(EditorNumberConstraints { min, max, step, .. }) => {
                AuthoringPropertyEditor::Slider {
                    min: min.unwrap_or(0.0),
                    max: max.unwrap_or(1.0),
                    step: step.unwrap_or(0.01),
                }
            }
            None => AuthoringPropertyEditor::Number,
        },
        EditorPropertyEditorKind::Checkbox => AuthoringPropertyEditor::Toggle,
        EditorPropertyEditorKind::EnumSelect => AuthoringPropertyEditor::Enum {
            options: descriptor
                .options
                .iter()
                .map(|option| option.id.to_owned())
                .collect(),
        },
        EditorPropertyEditorKind::AssetPicker => AuthoringPropertyEditor::AssetPicker {
            domain: descriptor
                .asset_domain
                .map(|domain| format!("{domain:?}"))
                .unwrap_or_else(|| "unknown".to_owned()),
        },
        EditorPropertyEditorKind::Vec2 => AuthoringPropertyEditor::Vec2,
        EditorPropertyEditorKind::Vec3 => AuthoringPropertyEditor::Vec3,
        EditorPropertyEditorKind::Color => AuthoringPropertyEditor::Color,
    }
}

fn hints_from_descriptor(descriptor: &EditorPropertyDescriptor) -> AuthoringPropertyHints {
    AuthoringPropertyHints {
        number: descriptor
            .number_constraints
            .map(|c| AuthoringNumberConstraints {
                min: c.min,
                max: c.max,
                step: c.step,
                clamp: c.clamp,
                unit: c.unit.map(str::to_owned),
                display_scale: c.display_scale,
            }),
        options: Vec::new(),
    }
}

fn value_from_descriptor(
    kind: EditorPropertyValueKind,
    value: Option<&Value>,
) -> AuthoringPropertyValue {
    match (kind, value) {
        (EditorPropertyValueKind::String, Some(Value::String(value)))
        | (EditorPropertyValueKind::AssetRef, Some(Value::String(value))) => {
            AuthoringPropertyValue::Text(value.clone())
        }
        (EditorPropertyValueKind::Enum, Some(Value::String(value))) => {
            AuthoringPropertyValue::Enum(value.clone())
        }
        (EditorPropertyValueKind::String | EditorPropertyValueKind::AssetRef, Some(value)) => {
            AuthoringPropertyValue::Text(short_yaml(value))
        }
        (EditorPropertyValueKind::Enum, Some(value)) => {
            AuthoringPropertyValue::Unsupported(short_yaml(value))
        }
        (EditorPropertyValueKind::Number, Some(value)) => value
            .as_f64()
            .map(|v| AuthoringPropertyValue::Number(v as f32))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        (EditorPropertyValueKind::Bool, Some(value)) => value
            .as_bool()
            .map(AuthoringPropertyValue::Bool)
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        (EditorPropertyValueKind::Vec2, Some(value)) => vec2_from_yaml(value)
            .map(|(x, y)| AuthoringPropertyValue::Vec2(x, y))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        (EditorPropertyValueKind::Vec3, Some(value)) => vec3_from_yaml(value)
            .map(|(x, y, z)| AuthoringPropertyValue::Vec3(x, y, z))
            .unwrap_or_else(|| AuthoringPropertyValue::Unsupported(short_yaml(value))),
        (EditorPropertyValueKind::Color, Some(value)) => {
            AuthoringPropertyValue::Unsupported(short_yaml(value))
        }
        (_, None) => AuthoringPropertyValue::Empty,
    }
}

fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |current, segment| {
        current.as_mapping()?.get(Value::String(segment.to_owned()))
    })
}

fn vec2_from_yaml(value: &Value) -> Option<(f32, f32)> {
    let mapping = value.as_mapping()?;
    let x = mapping.get(Value::String("x".to_owned()))?.as_f64()? as f32;
    let y = mapping.get(Value::String("y".to_owned()))?.as_f64()? as f32;
    Some((x, y))
}

fn vec3_from_yaml(value: &Value) -> Option<(f32, f32, f32)> {
    let mapping = value.as_mapping()?;
    let x = mapping.get(Value::String("x".to_owned()))?.as_f64()? as f32;
    let y = mapping.get(Value::String("y".to_owned()))?.as_f64()? as f32;
    let z = mapping.get(Value::String("z".to_owned()))?.as_f64()? as f32;
    Some((x, y, z))
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
