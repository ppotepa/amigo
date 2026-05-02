use std::fs;
use std::path::Path;

use serde_yaml::Value;

use crate::{PreviewColor, PreviewDrawItem, PreviewSceneInfo, PreviewSnapshot};

const SPRITE_COLOR: PreviewColor = PreviewColor::rgba(36, 123, 160, 210);
const CIRCLE_COLOR: PreviewColor = PreviewColor::rgba(112, 193, 179, 190);
const CAMERA_COLOR: PreviewColor = PreviewColor::rgba(255, 22, 84, 210);
const ENTITY_COLOR: PreviewColor = PreviewColor::rgba(44, 62, 80, 180);
const LABEL_COLOR: PreviewColor = PreviewColor::rgba(44, 62, 80, 255);

pub fn load_static_scene_preview(
    scene_path: impl AsRef<Path>,
    info: PreviewSceneInfo,
) -> Result<PreviewSnapshot, String> {
    let scene_path = scene_path.as_ref();
    let source = fs::read_to_string(scene_path)
        .map_err(|err| format!("Cannot read scene preview source `{}`: {err}", scene_path.display()))?;
    let document = serde_yaml::from_str::<Value>(&source)
        .map_err(|err| format!("Cannot parse scene preview source `{}`: {err}", scene_path.display()))?;

    let entities = document
        .get("entities")
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mut draw_items = Vec::new();
    for (index, entity) in entities.iter().enumerate() {
        collect_entity_draw_items(index, entity, &mut draw_items);
    }

    if draw_items.is_empty() {
        draw_items.push(PreviewDrawItem::Label {
            x: 0.0,
            y: 0.0,
            text: info.scene_label.clone(),
            color: LABEL_COLOR,
        });
    }

    Ok(PreviewSnapshot {
        info,
        source_path: Some(scene_path.display().to_string()),
        entities_count: entities.len(),
        draw_items,
    })
}

fn collect_entity_draw_items(index: usize, entity: &Value, draw_items: &mut Vec<PreviewDrawItem>) {
    let name = entity_string(entity, "name")
        .or_else(|| entity_string(entity, "id"))
        .unwrap_or_else(|| format!("entity-{index}"));
    let (x, y) = entity_translation(entity).unwrap_or_else(|| fallback_position(index));
    let components = entity
        .get("components")
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    if components.is_empty() {
        draw_items.push(PreviewDrawItem::Circle {
            x,
            y,
            r: 8.0,
            color: ENTITY_COLOR,
            label: Some(name),
        });
        return;
    }

    let mut emitted = false;
    for component in components {
        let kind = component
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match kind {
            "Sprite2D" | "InkDoodle2D" => {
                let (w, h) = vector2(&component, "size").unwrap_or((42.0, 42.0));
                draw_items.push(PreviewDrawItem::Rect {
                    x,
                    y,
                    w,
                    h,
                    color: SPRITE_COLOR,
                    label: Some(name.clone()),
                });
                emitted = true;
            }
            "CircleCollider2D" | "Circle2D" => {
                let radius = number(&component, "radius").unwrap_or(16.0);
                draw_items.push(PreviewDrawItem::Circle {
                    x,
                    y,
                    r: radius,
                    color: CIRCLE_COLOR,
                    label: Some(name.clone()),
                });
                emitted = true;
            }
            "Camera2D" => {
                draw_items.push(PreviewDrawItem::Rect {
                    x,
                    y,
                    w: 96.0,
                    h: 58.0,
                    color: CAMERA_COLOR,
                    label: Some(name.clone()),
                });
                emitted = true;
            }
            _ => {}
        }
    }

    if !emitted {
        draw_items.push(PreviewDrawItem::Circle {
            x,
            y,
            r: 6.0,
            color: ENTITY_COLOR,
            label: Some(name),
        });
    }
}

fn entity_string(entity: &Value, key: &str) -> Option<String> {
    entity.get(key).and_then(Value::as_str).map(String::from)
}

fn entity_translation(entity: &Value) -> Option<(f32, f32)> {
    let translation = entity.get("transform2")?.get("translation")?;
    Some((number(translation, "x")?, number(translation, "y")?))
}

fn vector2(value: &Value, key: &str) -> Option<(f32, f32)> {
    let value = value.get(key)?;
    Some((number(value, "x")?, number(value, "y")?))
}

fn number(value: &Value, key: &str) -> Option<f32> {
    value.get(key).and_then(value_to_f32)
}

fn value_to_f32(value: &Value) -> Option<f32> {
    value
        .as_f64()
        .map(|value| value as f32)
        .or_else(|| value.as_i64().map(|value| value as f32))
        .or_else(|| value.as_u64().map(|value| value as f32))
}

fn fallback_position(index: usize) -> (f32, f32) {
    let x = (index % 8) as f32 * 88.0 - 300.0;
    let y = (index / 8) as f32 * 72.0 - 180.0;
    (x, y)
}
