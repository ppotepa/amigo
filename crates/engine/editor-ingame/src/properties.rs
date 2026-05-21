use amigo_editor_authoring::{
    AuthoringPropertyEditor, AuthoringPropertyPanel, AuthoringPropertyValue,
    build_property_panel_for_node, build_property_panel_for_node_with_registry,
};
use amigo_runtime::Runtime;

use crate::state::EditorPropertyValue;

#[allow(dead_code)]
pub fn build_panel_with_overrides(
    node: &amigo_editor_authoring::AuthoringNode,
    override_for: impl Fn(&str) -> Option<EditorPropertyValue>,
) -> AuthoringPropertyPanel {
    let mut panel = build_property_panel_for_node(node);
    for group in &mut panel.groups {
        for property in &mut group.properties {
            if let Some(value) = override_for(&property.id) {
                property.value = display_value_from_override(value);
            }
        }
    }
    panel
}

pub fn build_panel_with_overrides_for_runtime(
    runtime: &Runtime,
    node: &amigo_editor_authoring::AuthoringNode,
    override_for: impl Fn(&str) -> Option<EditorPropertyValue>,
) -> AuthoringPropertyPanel {
    let registry = crate::component_registry::editor_component_registry(runtime);
    let mut panel = build_property_panel_for_node_with_registry(node, &registry);
    for group in &mut panel.groups {
        for property in &mut group.properties {
            if let Some(value) = override_for(&property.id) {
                property.value = display_value_from_override(value);
            }
        }
    }
    panel
}

pub fn as_number(value: &AuthoringPropertyValue) -> Option<f32> {
    match value {
        AuthoringPropertyValue::Number(v) => Some(*v),
        _ => None,
    }
}

pub fn as_bool(value: &AuthoringPropertyValue) -> Option<bool> {
    match value {
        AuthoringPropertyValue::Bool(v) => Some(*v),
        _ => None,
    }
}

pub fn display_text(label: &str, value: &AuthoringPropertyValue) -> String {
    match value {
        AuthoringPropertyValue::Text(v) => format!("{label}: {v}"),
        AuthoringPropertyValue::Number(v) => format!("{label}: {v:.3}"),
        AuthoringPropertyValue::Bool(v) => format!("{label}: {v}"),
        AuthoringPropertyValue::AssetRef(v) => format!("{label}: asset:{v}"),
        AuthoringPropertyValue::Enum(v) => format!("{label}: {v}"),
        AuthoringPropertyValue::Vec2(x, y) => format!("{label}: ({x:.3}, {y:.3})"),
        AuthoringPropertyValue::Vec3(x, y, z) => format!("{label}: ({x:.3}, {y:.3}, {z:.3})"),
        AuthoringPropertyValue::Color(v) => format!("{label}: {v}"),
        AuthoringPropertyValue::Empty => format!("{label}: <empty>"),
        AuthoringPropertyValue::Unsupported(v) => format!("{label}: {v}"),
    }
}

pub fn display_number_with_hints(
    value: f32,
    hints: &amigo_editor_authoring::AuthoringPropertyHints,
) -> String {
    if let Some(number) = &hints.number {
        let shown = value * number.display_scale;
        let unit = number.unit.as_deref().unwrap_or("");
        if unit.is_empty() {
            format!("{shown:.3}")
        } else {
            format!("{shown:.1}{unit}")
        }
    } else {
        format!("{value:.3}")
    }
}

pub fn is_slider(editor: &AuthoringPropertyEditor) -> Option<(f32, f32, f32)> {
    match editor {
        AuthoringPropertyEditor::Slider { min, max, step } => Some((*min, *max, *step)),
        _ => None,
    }
}

fn display_value_from_override(value: EditorPropertyValue) -> AuthoringPropertyValue {
    match value {
        EditorPropertyValue::Number(v) => AuthoringPropertyValue::Number(v),
        EditorPropertyValue::Bool(v) => AuthoringPropertyValue::Bool(v),
        EditorPropertyValue::Text(v) => AuthoringPropertyValue::Text(v),
        EditorPropertyValue::Enum(v) => AuthoringPropertyValue::Enum(v),
        EditorPropertyValue::Vec2(x, y) => AuthoringPropertyValue::Vec2(x, y),
        EditorPropertyValue::Vec3(x, y, z) => AuthoringPropertyValue::Vec3(x, y, z),
        EditorPropertyValue::Color(v) => AuthoringPropertyValue::Color(v),
        EditorPropertyValue::AssetRef(v) => AuthoringPropertyValue::AssetRef(v),
    }
}
