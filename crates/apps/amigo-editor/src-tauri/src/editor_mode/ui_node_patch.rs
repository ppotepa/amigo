use serde_yaml::{Mapping, Value};

use super::dto::EditorUiNodePropertyValueDto;

pub fn patch_ui_node_property(
    document: &mut Value,
    entity_id: &str,
    component_index: usize,
    node_path: &str,
    property_path: &str,
    value: EditorUiNodePropertyValueDto,
) -> Result<(), String> {
    let root = find_ui_document_root_mut(document, entity_id, component_index)?;
    let node = find_ui_node_mut(root, node_path)?;

    match property_path {
        "id" => set_optional_string(node, "id", value),
        "text" => set_optional_string(node, "text", value),
        "style_class" => set_optional_string(node, "style_class", value),
        "visible" => set_optional_bool(node, "visible", value),
        "enabled" => set_optional_bool(node, "enabled", value),
        path if path.starts_with("style.") => {
            let style_key = path.trim_start_matches("style.");
            set_style_property(node, style_key, value)
        }
        _ => Err(format!("unsupported UI node property `{property_path}`")),
    }
}

fn find_ui_document_root_mut<'a>(
    document: &'a mut Value,
    entity_id: &str,
    component_index: usize,
) -> Result<&'a mut Mapping, String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root is not a mapping".to_owned())?;
    let entities = root
        .get_mut(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "scene document has no entities sequence".to_owned())?;
    let entity = entities
        .iter_mut()
        .find(|entity| {
            entity
                .as_mapping()
                .and_then(|mapping| mapping.get(Value::String("id".to_owned())))
                .and_then(Value::as_str)
                == Some(entity_id)
        })
        .ok_or_else(|| format!("entity `{entity_id}` not found"))?;
    let entity = entity
        .as_mapping_mut()
        .ok_or_else(|| format!("entity `{entity_id}` is not a mapping"))?;
    let components = entity
        .get_mut(Value::String("components".to_owned()))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| format!("entity `{entity_id}` has no components"))?;
    let component = components.get_mut(component_index).ok_or_else(|| {
        format!("component index `{component_index}` out of range for entity `{entity_id}`")
    })?;
    let component = component.as_mapping_mut().ok_or_else(|| {
        format!("component `{component_index}` on entity `{entity_id}` is not a mapping")
    })?;
    let component_type = component
        .get(Value::String("type".to_owned()))
        .and_then(Value::as_str)
        .unwrap_or_default();

    if component_type != "UiDocument" {
        return Err(format!(
            "component `{component_index}` on entity `{entity_id}` is `{component_type}`, expected UiDocument"
        ));
    }

    component
        .get_mut(Value::String("root".to_owned()))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| {
            format!("UiDocument component `{component_index}` on entity `{entity_id}` has no root")
        })
}

fn find_ui_node_mut<'a>(root: &'a mut Mapping, node_path: &str) -> Result<&'a mut Mapping, String> {
    let parts = node_path
        .split('.')
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return Err("UI node path is empty".to_owned());
    }

    let root_id = string_field(root, "id").unwrap_or_else(|| "root".to_owned());
    if parts[0] != root_id {
        return Err(format!(
            "UI node path `{node_path}` does not start at root id `{root_id}`"
        ));
    }

    find_ui_node_parts_mut(root, &parts[1..], node_path)
}

fn find_ui_node_parts_mut<'a>(
    current: &'a mut Mapping,
    remaining: &[&str],
    full_path: &str,
) -> Result<&'a mut Mapping, String> {
    if remaining.is_empty() {
        return Ok(current);
    }

    let target_id = remaining[0];
    let children = current
        .get_mut(Value::String("children".to_owned()))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| format!("UI node `{full_path}` cannot be found; missing children"))?;
    let child = children
        .iter_mut()
        .find(|child| {
            child
                .as_mapping()
                .and_then(|mapping| mapping.get(Value::String("id".to_owned())))
                .and_then(Value::as_str)
                == Some(target_id)
        })
        .ok_or_else(|| format!("UI node path segment `{target_id}` not found in `{full_path}`"))?;
    let child = child
        .as_mapping_mut()
        .ok_or_else(|| format!("UI node path segment `{target_id}` is not a mapping"))?;

    find_ui_node_parts_mut(child, &remaining[1..], full_path)
}

fn set_optional_string(
    node: &mut Mapping,
    key: &str,
    value: EditorUiNodePropertyValueDto,
) -> Result<(), String> {
    match value {
        EditorUiNodePropertyValueDto::String(value) => {
            node.insert(Value::String(key.to_owned()), Value::String(value));
            Ok(())
        }
        EditorUiNodePropertyValueDto::Null => {
            node.remove(Value::String(key.to_owned()));
            Ok(())
        }
        other => Err(format!("expected string/null for `{key}`, got `{other:?}`")),
    }
}

fn set_optional_bool(
    node: &mut Mapping,
    key: &str,
    value: EditorUiNodePropertyValueDto,
) -> Result<(), String> {
    match value {
        EditorUiNodePropertyValueDto::Bool(value) => {
            node.insert(Value::String(key.to_owned()), Value::Bool(value));
            Ok(())
        }
        EditorUiNodePropertyValueDto::Null => {
            node.remove(Value::String(key.to_owned()));
            Ok(())
        }
        other => Err(format!("expected bool/null for `{key}`, got `{other:?}`")),
    }
}

fn set_style_property(
    node: &mut Mapping,
    key: &str,
    value: EditorUiNodePropertyValueDto,
) -> Result<(), String> {
    let style_key = Value::String("style".to_owned());
    if matches!(value, EditorUiNodePropertyValueDto::Null) {
        if let Some(style) = node.get_mut(&style_key).and_then(Value::as_mapping_mut) {
            style.remove(Value::String(key.to_owned()));
        }
        return Ok(());
    }

    if !node.contains_key(&style_key) {
        node.insert(style_key.clone(), Value::Mapping(Mapping::new()));
    }

    let style = node
        .get_mut(&style_key)
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| "UI node style exists but is not a mapping".to_owned())?;
    let yaml_value = match value {
        EditorUiNodePropertyValueDto::String(value) => Value::String(value),
        EditorUiNodePropertyValueDto::Number(value) => {
            serde_yaml::to_value(value).map_err(|error| error.to_string())?
        }
        EditorUiNodePropertyValueDto::Bool(value) => Value::Bool(value),
        EditorUiNodePropertyValueDto::Null => unreachable!(),
    };
    style.insert(Value::String(key.to_owned()), yaml_value);
    Ok(())
}

fn string_field(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_button_text_inside_ui_document() {
        let mut document = serde_yaml::from_str::<Value>(
            r#"
version: 1
scene: { id: main-menu }
entities:
  - id: main-menu-ui
    components:
      - type: UiDocument
        root:
          type: column
          id: root
          children:
            - type: panel
              id: menu-card
              children:
                - type: button
                  id: start
                  text: START
"#,
        )
        .unwrap();

        patch_ui_node_property(
            &mut document,
            "main-menu-ui",
            0,
            "root.menu-card.start",
            "text",
            EditorUiNodePropertyValueDto::String("BEGIN CASE".to_owned()),
        )
        .unwrap();

        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("BEGIN CASE"));
    }

    #[test]
    fn patches_button_style_inside_ui_document() {
        let mut document = serde_yaml::from_str::<Value>(
            r#"
version: 1
scene: { id: main-menu }
entities:
  - id: main-menu-ui
    components:
      - type: UiDocument
        root:
          type: column
          id: root
          children:
            - type: button
              id: start
              text: START
"#,
        )
        .unwrap();

        patch_ui_node_property(
            &mut document,
            "main-menu-ui",
            0,
            "root.start",
            "style.width",
            EditorUiNodePropertyValueDto::Number(420.0),
        )
        .unwrap();

        let text = serde_yaml::to_string(&document).unwrap();
        assert!(text.contains("width: 420"));
    }
}
