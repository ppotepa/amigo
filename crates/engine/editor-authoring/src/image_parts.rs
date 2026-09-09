use amigo_editor_api::{
    AuthoringNumberConstraints, AuthoringProperty, AuthoringPropertyApplyMode,
    AuthoringPropertyDisplay, AuthoringPropertyEditor, AuthoringPropertyHints,
    AuthoringPropertyValue, AuthoringPropertyVisibility, AuthoringRuntimeBinding,
};
use serde_yaml::Value;

use crate::AuthoringNode;

#[derive(Debug, Clone, PartialEq)]
pub struct ImagePartDescriptor {
    pub id: String,
    pub opacity: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePartField {
    Opacity,
    Enabled,
}

#[derive(Debug, Clone, Copy)]
pub struct ImagePartPropertyDescriptor {
    pub field: ImagePartField,
    pub label_suffix: &'static str,
    pub group: &'static str,
    pub order: i32,
}

pub const IMAGE_PART_PROPERTY_DESCRIPTORS: &[ImagePartPropertyDescriptor] = &[
    ImagePartPropertyDescriptor {
        field: ImagePartField::Opacity,
        label_suffix: "opacity",
        group: "render2d.image_parts.dynamic",
        order: 0,
    },
    ImagePartPropertyDescriptor {
        field: ImagePartField::Enabled,
        label_suffix: "enabled",
        group: "render2d.image_parts.dynamic",
        order: 1,
    },
];

pub struct ImagePartRuntimeBindingResolver;

impl ImagePartRuntimeBindingResolver {
    pub fn opacity(entity_name: &str, part_id: &str) -> AuthoringRuntimeBinding {
        AuthoringRuntimeBinding::LayeredImageLayerOpacity {
            entity_name: entity_name.to_owned(),
            layer_id: part_id.to_owned(),
        }
    }

    pub fn enabled(entity_name: &str, part_id: &str) -> AuthoringRuntimeBinding {
        AuthoringRuntimeBinding::LayeredImageLayerEnabled {
            entity_name: entity_name.to_owned(),
            layer_id: part_id.to_owned(),
        }
    }
}

pub fn collect_image_part_properties(node: &AuthoringNode) -> Vec<AuthoringProperty> {
    let entity_name = node
        .semantic
        .owner_entity_name
        .clone()
        .unwrap_or_else(|| "unknown-entity".to_owned());

    let Some(overrides) = mapping_get(&node.value, "layer_overrides").and_then(Value::as_sequence)
    else {
        return Vec::new();
    };

    let mut properties = Vec::new();
    for item in overrides {
        let Some(part_id) = string_field(item, "id") else {
            continue;
        };
        let part = ImagePartDescriptor {
            id: part_id,
            opacity: number_field(item, "opacity").unwrap_or(1.0),
            enabled: bool_field(item, "enabled").unwrap_or(true),
        };
        for descriptor in IMAGE_PART_PROPERTY_DESCRIPTORS {
            properties.push(property_from_image_part_descriptor(
                node,
                &entity_name,
                &part,
                descriptor,
            ));
        }
    }
    properties
}

fn property_from_image_part_descriptor(
    node: &AuthoringNode,
    entity_name: &str,
    part: &ImagePartDescriptor,
    descriptor: &ImagePartPropertyDescriptor,
) -> AuthoringProperty {
    match descriptor.field {
        ImagePartField::Opacity => number_property(
            node,
            descriptor,
            &part.id,
            part.opacity,
            ImagePartRuntimeBindingResolver::opacity(entity_name, &part.id),
        ),
        ImagePartField::Enabled => bool_property(
            node,
            descriptor,
            &part.id,
            part.enabled,
            ImagePartRuntimeBindingResolver::enabled(entity_name, &part.id),
        ),
    }
}

fn number_property(
    node: &AuthoringNode,
    descriptor: &ImagePartPropertyDescriptor,
    part_id: &str,
    value: f32,
    binding: AuthoringRuntimeBinding,
) -> AuthoringProperty {
    let path = image_part_path(part_id, descriptor.field);
    AuthoringProperty {
        id: format!("{}::{}", node.id, path),
        label: format!("{part_id} {}", descriptor.label_suffix),
        value: AuthoringPropertyValue::Number(value),
        editor: AuthoringPropertyEditor::Slider {
            min: 0.0,
            max: 1.0,
            step: 0.01,
        },
        hints: AuthoringPropertyHints {
            number: Some(AuthoringNumberConstraints {
                min: Some(0.0),
                max: Some(1.0),
                step: Some(0.01),
                clamp: true,
                unit: None,
                display_scale: 1.0,
            }),
            options: Vec::new(),
        },
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        // Image-part fields currently identify the component rather than the
        // nested list element. Do not offer a stale source save until that
        // structural path has a precise pointer.
        source_value: None,
        group: descriptor.group.to_owned(),
        trait_kind: None,
        binding: Some(binding),
        display: live_display(
            vec!["ImagePart".to_owned(), "Slider".to_owned()],
            descriptor.order,
        ),
    }
}

fn bool_property(
    node: &AuthoringNode,
    descriptor: &ImagePartPropertyDescriptor,
    part_id: &str,
    value: bool,
    binding: AuthoringRuntimeBinding,
) -> AuthoringProperty {
    let path = image_part_path(part_id, descriptor.field);
    AuthoringProperty {
        id: format!("{}::{}", node.id, path),
        label: format!("{part_id} {}", descriptor.label_suffix),
        value: AuthoringPropertyValue::Bool(value),
        editor: AuthoringPropertyEditor::Toggle,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: node.yaml_pointer.clone(),
        source_value: None,
        group: descriptor.group.to_owned(),
        trait_kind: None,
        binding: Some(binding),
        display: live_display(
            vec!["ImagePart".to_owned(), "Toggle".to_owned()],
            descriptor.order,
        ),
    }
}

fn live_display(mut tags: Vec<String>, order: i32) -> AuthoringPropertyDisplay {
    tags.push("Live".to_owned());
    AuthoringPropertyDisplay {
        icon: None,
        tags,
        visibility: AuthoringPropertyVisibility::Primary,
        apply_mode: AuthoringPropertyApplyMode::Live,
        order,
    }
}

fn image_part_path(part_id: &str, field: ImagePartField) -> String {
    let field = match field {
        ImagePartField::Opacity => "opacity",
        ImagePartField::Enabled => "enabled",
    };
    format!("layer_overrides.{part_id}.{field}")
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
