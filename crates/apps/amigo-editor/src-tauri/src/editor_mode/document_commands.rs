use std::fs;
use std::path::Path;

use serde_yaml::{Mapping, Value};

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::document_snapshot::{document_editor_snapshot, scene_document_path};
use crate::editor_mode::dto::{EditorCommandResultDto, EditorTransform2Dto};

pub fn apply_document_transform_2d(
    mod_id: String,
    root_path: impl AsRef<Path>,
    scene_id: String,
    entity_id: String,
    transform: EditorTransform2Dto,
) -> Result<EditorCommandResultDto, String> {
    let scene_path = scene_document_path(root_path.as_ref(), &scene_id);
    let text = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "failed to read scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    let mut value = serde_yaml::from_str::<Value>(&text).map_err(|error| {
        format!(
            "failed to parse scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    assert_entity_has_transform_2d(&value, &entity_id)?;
    patch_entity_transform_2d(&mut value, &entity_id, &transform)?;

    let next_text = serde_yaml::to_string(&value).map_err(|error| {
        format!(
            "failed to serialize scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    fs::write(&scene_path, &next_text).map_err(|error| {
        format!(
            "failed to write scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    let snapshot = match document_editor_snapshot(mod_id, root_path.as_ref(), scene_id) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let _ = fs::write(&scene_path, text);
            return Ok(EditorCommandResultDto {
                ok: false,
                scene_dirty: false,
                changed_entities: vec![entity_id],
                snapshot: None,
                diagnostics: vec![EditorDiagnosticDto {
                    level: DiagnosticLevel::Error,
                    code: "DOCUMENT_VALIDATION_FAILED_AFTER_PATCH".to_owned(),
                    message: format!(
                        "Transform patch was reverted because the scene document failed validation: {error}"
                    ),
                    path: Some(scene_path.display().to_string()),
                }],
                message: Some("Transform patch was reverted after failed validation.".to_owned()),
            });
        }
    };

    Ok(EditorCommandResultDto {
        ok: true,
        scene_dirty: true,
        changed_entities: vec![entity_id],
        snapshot: Some(snapshot),
        diagnostics: Vec::new(),
        message: Some("Scene document transform2 was updated.".to_owned()),
    })
}

fn assert_entity_has_transform_2d(value: &Value, entity_id: &str) -> Result<(), String> {
    let entities = entities_sequence(value)?;
    for entity_value in entities {
        let Some(entity) = entity_value.as_mapping() else {
            continue;
        };
        let id = entity
            .get(Value::String("id".to_owned()))
            .and_then(Value::as_str);
        if id != Some(entity_id) {
            continue;
        }

        if entity.get(Value::String("transform2".to_owned())).is_some() {
            return Ok(());
        }

        return Err(format!(
            "ENTITY_NO_TRANSFORM2: entity `{entity_id}` has no transform2 block; create it explicitly before using viewport transform editing"
        ));
    }

    Err(format!(
        "ENTITY_NOT_FOUND: entity `{entity_id}` was not found in scene document"
    ))
}

fn entities_sequence(value: &Value) -> Result<&Vec<Value>, String> {
    let root = value
        .as_mapping()
        .ok_or_else(|| "scene document root is not a mapping".to_owned())?;
    root.get(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence)
        .or_else(|| {
            root.get(Value::String("scene".to_owned()))
                .and_then(Value::as_mapping)
                .and_then(|scene| {
                    scene
                        .get(Value::String("entities".to_owned()))
                        .and_then(Value::as_sequence)
                })
        })
        .ok_or_else(|| "scene document has no entities array".to_owned())
}

fn patch_entity_transform_2d(
    value: &mut Value,
    entity_id: &str,
    transform: &EditorTransform2Dto,
) -> Result<(), String> {
    let root = value
        .as_mapping_mut()
        .ok_or_else(|| "scene document root is not a mapping".to_owned())?;
    let entities = if let Some(entities) = root
        .get_mut(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence_mut)
    {
        entities
    } else if let Some(entities) = root
        .get_mut(Value::String("scene".to_owned()))
        .and_then(Value::as_mapping_mut)
        .and_then(|scene| {
            scene
                .get_mut(Value::String("entities".to_owned()))
                .and_then(Value::as_sequence_mut)
        })
    {
        entities
    } else {
        return Err("scene document has no entities array".to_owned());
    };

    for entity_value in entities {
        let Some(entity) = entity_value.as_mapping_mut() else {
            continue;
        };
        let id = entity
            .get(Value::String("id".to_owned()))
            .and_then(Value::as_str);
        if id != Some(entity_id) {
            continue;
        }

        let transform_value = entity
            .get_mut(Value::String("transform2".to_owned()))
            .ok_or_else(|| {
                format!("ENTITY_NO_TRANSFORM2: entity `{entity_id}` has no transform2 block")
            })?;
        let transform_map = transform_value
            .as_mapping_mut()
            .ok_or_else(|| format!("entity `{entity_id}` transform2 is not a mapping"))?;

        write_vec2(transform_map, "translation", transform.x, transform.y);
        write_vec2(transform_map, "scale", transform.scale_x, transform.scale_y);
        transform_map.insert(
            Value::String("rotation_radians".to_owned()),
            number_value(transform.rotation),
        );

        if let Some(z_index) = transform.z_index {
            transform_map.insert(
                Value::String("z_index".to_owned()),
                Value::Number(serde_yaml::Number::from(z_index)),
            );
        }

        return Ok(());
    }

    Err(format!(
        "entity `{entity_id}` was not found in scene document"
    ))
}

fn write_vec2(parent: &mut Mapping, key: &str, x: f32, y: f32) {
    let mut value = Mapping::new();
    value.insert(Value::String("x".to_owned()), number_value(x));
    value.insert(Value::String("y".to_owned()), number_value(y));
    parent.insert(Value::String(key.to_owned()), Value::Mapping(value));
}

fn number_value(value: f32) -> Value {
    serde_yaml::to_value(value).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_entity_transform_2d_updates_existing_transform() {
        let mut value = serde_yaml::from_str::<Value>(
            r#"
version: 1
scene: { id: test }
entities:
  - id: player
    transform2:
      translation: { x: 1.0, y: 2.0 }
      rotation_radians: 0.0
      scale: { x: 1.0, y: 1.0 }
"#,
        )
        .unwrap();

        patch_entity_transform_2d(
            &mut value,
            "player",
            &EditorTransform2Dto {
                x: 10.0,
                y: 20.0,
                rotation: 0.25,
                scale_x: 2.0,
                scale_y: 3.0,
                z_index: Some(7),
            },
        )
        .unwrap();

        let text = serde_yaml::to_string(&value).unwrap();
        assert!(text.contains("x: 10.0") || text.contains("x: 10"));
        assert!(text.contains("y: 20.0") || text.contains("y: 20"));
        assert!(text.contains("rotation_radians"));
        assert!(text.contains("z_index: 7"));
    }

    #[test]
    fn patch_entity_transform_2d_rejects_entity_without_transform() {
        let mut value = serde_yaml::from_str::<Value>(
            r#"
version: 1
scene: { id: test }
entities:
  - id: controller
    components:
      - type: Behavior
"#,
        )
        .unwrap();

        let result = patch_entity_transform_2d(
            &mut value,
            "controller",
            &EditorTransform2Dto {
                x: 10.0,
                y: 20.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                z_index: None,
            },
        );

        assert!(result.unwrap_err().contains("ENTITY_NO_TRANSFORM2"));
    }
}
