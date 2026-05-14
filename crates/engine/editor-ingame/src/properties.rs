use amigo_editor_authoring::{
    AuthoringPropertyEditor, AuthoringPropertyPanel, AuthoringPropertyValue,
    build_property_panel_for_node,
};

use crate::state::EditorPropertyValue;

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
    }
}
