fn postfx_panel(node: &AuthoringNode) -> AuthoringPropertyPanel {
    let effect_type = string_field(&node.value, "type").unwrap_or_else(|| "effect".to_owned());
    let effect_id = string_field(&node.value, "id").unwrap_or_else(|| effect_type.clone());
    let mut groups = vec![AuthoringPropertyGroup {
        id: "postfx".to_owned(),
        title: "PostFX".to_owned(),
        properties: vec![
            status_text_primary(node, "kind", effect_type.as_str()),
            readonly_text(node, "id", effect_id.as_str()),
            readonly_text(
                node,
                "scope",
                node.semantic
                    .post_fx_scope
                    .clone()
                    .unwrap_or_else(|| "frame".to_owned()),
            ),
        ],
    }];

    let Some(index) = postfx_frame_index(node) else {
        groups.push(AuthoringPropertyGroup {
            id: "status".to_owned(),
            title: "Status".to_owned(),
            properties: vec![readonly_text(
                node,
                "binding",
                "No frame index; live binding unavailable",
            )],
        });
        return AuthoringPropertyPanel {
            title: format!("Frame Post FX: {effect_id}"),
            groups,
        };
    };

    groups[0].properties.push(AuthoringProperty {
        id: format!("{}::enabled", node.id),
        label: "Enabled".to_owned(),
        value: AuthoringPropertyValue::Bool(bool_field(&node.value, "enabled").unwrap_or(true)),
        editor: AuthoringPropertyEditor::Toggle,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: child_pointer(&node.yaml_pointer, "enabled"),
        group: "postfx".to_owned(),
        trait_kind: None,
        binding: Some(AuthoringRuntimeBinding::PostFxFrameEnabled { index }),
        display: AuthoringPropertyDisplay {
            apply_mode: AuthoringPropertyApplyMode::Live,
            ..Default::default()
        },
    });

    if effect_type == "rain_glass" {
        groups.push(AuthoringPropertyGroup {
            id: "optics".to_owned(),
            title: "Optics".to_owned(),
            properties: vec![
                postfx_float(node, index, "opacity", 0.0, 1.0, 0.01),
                postfx_float(node, index, "refract_scale", 0.0, 3.0, 0.01),
                postfx_float(node, index, "background_blur_px", 0.0, 16.0, 0.1),
                postfx_float(node, index, "distortion_px", 0.0, 64.0, 0.1),
                postfx_float(node, index, "normal_strength", 0.0, 4.0, 0.01),
                postfx_float(node, index, "focus_blur_strength", 0.0, 2.0, 0.01),
                postfx_float(node, index, "body_opacity", 0.0, 1.0, 0.01),
                postfx_float(node, index, "scene_blend", 0.0, 1.0, 0.01),
            ],
        });
        groups.push(AuthoringPropertyGroup {
            id: "mist".to_owned(),
            title: "Mist".to_owned(),
            properties: vec![
                postfx_toggle(node, index, "trails_enabled"),
                postfx_toggle(node, index, "mist_enabled"),
                postfx_float(node, index, "mist_opacity", 0.0, 1.0, 0.01),
                postfx_debug_view(node, index),
            ],
        });
    }

    AuthoringPropertyPanel {
        title: format!("Frame Post FX: {effect_id}"),
        groups,
    }
}

fn postfx_frame_index(node: &AuthoringNode) -> Option<usize> {
    node.yaml_pointer
        .rsplit('/')
        .find_map(|segment| segment.parse::<usize>().ok())
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    mapping_get(value, key)?.as_bool()
}

fn float_field(value: &Value, key: &str) -> Option<f32> {
    mapping_get(value, key)?.as_f64().map(|value| value as f32)
}

fn postfx_float(
    node: &AuthoringNode,
    index: usize,
    field: &str,
    min: f32,
    max: f32,
    step: f32,
) -> AuthoringProperty {
    AuthoringProperty {
        id: format!("{}::{field}", node.id),
        label: field.replace('_', " "),
        value: AuthoringPropertyValue::Number(float_field(&node.value, field).unwrap_or(min)),
        editor: AuthoringPropertyEditor::Slider { min, max, step },
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: child_pointer(&node.yaml_pointer, field),
        group: "postfx".to_owned(),
        trait_kind: None,
        binding: Some(AuthoringRuntimeBinding::PostFxFrameField {
            index,
            field: field.to_owned(),
        }),
        display: AuthoringPropertyDisplay {
            apply_mode: AuthoringPropertyApplyMode::Live,
            ..Default::default()
        },
    }
}

fn postfx_toggle(node: &AuthoringNode, index: usize, field: &str) -> AuthoringProperty {
    AuthoringProperty {
        id: format!("{}::{field}", node.id),
        label: field.replace('_', " "),
        value: AuthoringPropertyValue::Bool(bool_field(&node.value, field).unwrap_or(false)),
        editor: AuthoringPropertyEditor::Toggle,
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: child_pointer(&node.yaml_pointer, field),
        group: "postfx".to_owned(),
        trait_kind: None,
        binding: Some(AuthoringRuntimeBinding::PostFxFrameField {
            index,
            field: field.to_owned(),
        }),
        display: AuthoringPropertyDisplay {
            apply_mode: AuthoringPropertyApplyMode::Live,
            ..Default::default()
        },
    }
}

fn postfx_debug_view(node: &AuthoringNode, index: usize) -> AuthoringProperty {
    let value = string_field(&node.value, "debug_view").unwrap_or_else(|| "Final".to_owned());
    AuthoringProperty {
        id: format!("{}::debug_view", node.id),
        label: "debug view".to_owned(),
        value: AuthoringPropertyValue::Enum(value),
        editor: AuthoringPropertyEditor::Enum {
            options: vec![
                "Final".to_owned(),
                "SceneInput".to_owned(),
                "BlurredScene".to_owned(),
                "RaindropMap".to_owned(),
                "DropletMap".to_owned(),
                "DropNormals".to_owned(),
                "DropMask".to_owned(),
                "Mist".to_owned(),
                "Refraction".to_owned(),
            ],
        },
        hints: AuthoringPropertyHints::default(),
        read_only: false,
        source_file: node.source_file.display().to_string(),
        yaml_pointer: child_pointer(&node.yaml_pointer, "debug_view"),
        group: "postfx".to_owned(),
        trait_kind: None,
        binding: Some(AuthoringRuntimeBinding::PostFxFrameField {
            index,
            field: "debug_view".to_owned(),
        }),
        display: AuthoringPropertyDisplay {
            apply_mode: AuthoringPropertyApplyMode::Live,
            ..Default::default()
        },
    }
}

