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

