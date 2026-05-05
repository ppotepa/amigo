use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::dto::{
    EditorBounds2Dto, EditorCameraDto, EditorSceneCanvasKindDto, EditorSceneObjectDto,
    EditorSceneSnapshotDto, EditorSceneSnapshotLayoutSourceDto, EditorSceneSnapshotQualityDto,
    EditorTransform2Dto, EditorTransform3Dto,
};

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;
const ICON_BOUNDS_SIZE: f32 = 32.0;
const DIAG_ENTITY_SKIPPED: &str = "EDITOR_MODE_ENTITY_SKIPPED";
const DIAG_NO_DOCUMENT_BOUNDS: &str = "EDITOR_MODE_NO_DOCUMENT_BOUNDS";
const DIAG_ENTITY_NO_TRANSFORM2: &str = "ENTITY_NO_TRANSFORM2";
const DIAG_ENTITY_NO_BOUNDS2: &str = "ENTITY_NO_BOUNDS2";
const DIAG_COMPONENT_BOUNDS_UNSUPPORTED: &str = "COMPONENT_BOUNDS_UNSUPPORTED";
const DIAG_DOCUMENT_PARSE_FAILED: &str = "DOCUMENT_PARSE_FAILED";

pub fn document_editor_snapshot(
    mod_id: String,
    root_path: impl AsRef<Path>,
    scene_id: String,
) -> Result<EditorSceneSnapshotDto, String> {
    let scene_path = scene_document_path(root_path.as_ref(), &scene_id);
    let text = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "failed to read scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    let value = serde_yaml::from_str::<Value>(&text).map_err(|error| {
        format!(
            "{DIAG_DOCUMENT_PARSE_FAILED}: failed to parse scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    snapshot_from_scene_value(mod_id, scene_id, &value)
}

pub fn snapshot_from_scene_value(
    mod_id: String,
    scene_id: String,
    value: &Value,
) -> Result<EditorSceneSnapshotDto, String> {
    let entities =
        scene_entities(value).ok_or_else(|| "scene document has no entities array".to_owned())?;
    let mut diagnostics = Vec::new();
    let mut objects = Vec::new();

    for (index, entity_value) in entities.iter().enumerate() {
        match object_from_entity_value(entity_value) {
            Some(object) => objects.push(object),
            None => push_diagnostic(
                &mut diagnostics,
                DiagnosticLevel::Info,
                DIAG_ENTITY_SKIPPED,
                format!(
                    "Scene entity at index {index} could not be converted into an editor object."
                ),
            ),
        }
    }
    append_object_quality_diagnostics(&objects, &mut diagnostics);

    if objects.is_empty() {
        push_diagnostic(
            &mut diagnostics,
            DiagnosticLevel::Info,
            DIAG_NO_DOCUMENT_BOUNDS,
            "Scene document was parsed, but no entity exposes transform/bounds data yet.",
        );
    }

    let canvas_kind = infer_canvas_kind_from_objects(&objects);
    Ok(EditorSceneSnapshotDto {
        mod_id,
        scene_id,
        canvas_kind,
        layout_source: EditorSceneSnapshotLayoutSourceDto::Document,
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        camera: EditorCameraDto {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            viewport_width: DEFAULT_WIDTH as f32,
            viewport_height: DEFAULT_HEIGHT as f32,
        },
        quality: snapshot_quality(entities.len(), &objects, &diagnostics),
        objects,
        diagnostics,
    })
}

pub fn scene_document_path(root_path: &Path, scene_id: &str) -> PathBuf {
    root_path.join("scenes").join(scene_id).join("scene.yml")
}

fn scene_entities(value: &Value) -> Option<&Vec<Value>> {
    let root = value.as_mapping()?;
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
}

fn object_from_entity_value(entity_value: &Value) -> Option<EditorSceneObjectDto> {
    let entity = mapping(entity_value)?;
    let entity_id = string_field(entity, "id")?;
    let name = string_field(entity, "name").unwrap_or_else(|| entity_id.clone());
    let visible = bool_field(entity, "visible").unwrap_or(true);
    let components = entity
        .get(Value::String("components".to_owned()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let component_types = components
        .iter()
        .filter_map(component_type)
        .collect::<Vec<_>>();
    let transform_2 = entity
        .get(Value::String("transform2".to_owned()))
        .and_then(transform2_from_value);
    let transform_3 = entity
        .get(Value::String("transform3".to_owned()))
        .and_then(transform3_from_value);
    let category = object_category(&component_types, transform_3.is_some());
    let bounds_2 = bounds2_for_entity(&components, transform_2.as_ref(), &category);
    let selectable = visible && bounds_2.is_some() && transform_2.is_some();

    Some(EditorSceneObjectDto {
        entity_id,
        name,
        visible,
        selectable,
        locked: !selectable,
        category,
        component_types,
        transform_2,
        transform_3,
        bounds_2,
    })
}

fn component_type(component: &Value) -> Option<String> {
    let component = mapping(component)?;
    string_field(component, "type").or_else(|| string_field(component, "kind"))
}

fn transform2_from_value(value: &Value) -> Option<EditorTransform2Dto> {
    let transform = mapping(value)?;
    let translation = transform
        .get(Value::String("translation".to_owned()))
        .and_then(mapping);
    let scale = transform
        .get(Value::String("scale".to_owned()))
        .and_then(mapping);

    Some(EditorTransform2Dto {
        x: translation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: translation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation: number_field(transform, "rotation_radians")
            .or_else(|| number_field(transform, "rotation"))
            .unwrap_or(0.0),
        scale_x: scale
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(1.0),
        scale_y: scale
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(1.0),
        z_index: number_field(transform, "z_index").map(|value| value as i32),
    })
}

fn transform3_from_value(value: &Value) -> Option<EditorTransform3Dto> {
    let transform = mapping(value)?;
    let translation = transform
        .get(Value::String("translation".to_owned()))
        .and_then(mapping);
    let rotation = transform
        .get(Value::String("rotation".to_owned()))
        .and_then(mapping);
    let scale = transform
        .get(Value::String("scale".to_owned()))
        .and_then(mapping);

    Some(EditorTransform3Dto {
        x: translation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: translation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        z: translation
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(0.0),
        rotation_x: rotation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        rotation_y: rotation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation_z: rotation
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(0.0),
        scale_x: scale
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(1.0),
        scale_y: scale
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(1.0),
        scale_z: scale
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(1.0),
    })
}

fn bounds2_for_entity(
    components: &[Value],
    transform: Option<&EditorTransform2Dto>,
    category: &str,
) -> Option<EditorBounds2Dto> {
    let transform = transform?;
    let mut best_size = None;

    for component in components {
        let Some(kind) = component_type(component) else {
            continue;
        };
        let Some(component) = mapping(component) else {
            continue;
        };

        let candidate = match kind.as_str() {
            "Sprite2D" | "sprite2d" | "sprite" => size_from_component(component)
                .or_else(|| sheet_frame_size(component))
                .or(Some((96.0, 96.0))),
            "Text2D" | "text2d" | "text" => bounds_size_from_component(component)
                .or_else(|| estimated_text_size(component))
                .or(Some((180.0, 42.0))),
            "Vector2D" | "VectorShape2D" | "vector2d" | "vector" => {
                size_from_component(component).or(Some((96.0, 96.0)))
            }
            "TileMap2D" | "Tilemap2D" | "tilemap2d" | "tilemap" => {
                tilemap_size_from_component(component)
            }
            "Bounds2D" | "bounds2d" => size_from_component(component),
            "Camera2D" | "camera2d" | "Light2D" | "light2d" => {
                Some((ICON_BOUNDS_SIZE, ICON_BOUNDS_SIZE))
            }
            "UiNode" | "UI" | "ui" | "Button" | "button" => {
                ui_size_from_component(component).or(Some((180.0, 56.0)))
            }
            _ => None,
        };

        if candidate.is_some() {
            best_size = candidate;
            break;
        }
    }

    if best_size.is_none() && matches!(category, "camera" | "light" | "script") {
        best_size = Some((ICON_BOUNDS_SIZE, ICON_BOUNDS_SIZE));
    }

    let (width, height) = best_size?;
    Some(EditorBounds2Dto {
        x: transform.x - width / 2.0,
        y: transform.y - height / 2.0,
        width,
        height,
    })
}

fn ui_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    size_from_component(component)
        .or_else(|| bounds_size_from_component(component))
        .or_else(|| rect_size_from_component(component))
}

fn size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("size".to_owned()))
        .and_then(mapping)
        .and_then(|size| Some((number_field(size, "x")?, number_field(size, "y")?)))
}

fn bounds_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("bounds".to_owned()))
        .and_then(mapping)
        .and_then(|bounds| Some((number_field(bounds, "x")?, number_field(bounds, "y")?)))
}

fn rect_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("rect".to_owned()))
        .and_then(mapping)
        .and_then(|rect| Some((number_field(rect, "width")?, number_field(rect, "height")?)))
}

fn sheet_frame_size(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("sheet".to_owned()))
        .and_then(mapping)
        .and_then(|sheet| {
            sheet
                .get(Value::String("frame_size".to_owned()))
                .and_then(mapping)
        })
        .and_then(|frame| Some((number_field(frame, "x")?, number_field(frame, "y")?)))
}

fn tilemap_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    let columns =
        number_field(component, "columns").or_else(|| number_field(component, "width"))?;
    let rows = number_field(component, "rows").or_else(|| number_field(component, "height"))?;
    let cell_size = component
        .get(Value::String("cell_size".to_owned()))
        .or_else(|| component.get(Value::String("tile_size".to_owned())))
        .and_then(mapping);
    let cell_width = cell_size
        .and_then(|value| number_field(value, "x"))
        .unwrap_or(32.0);
    let cell_height = cell_size
        .and_then(|value| number_field(value, "y"))
        .unwrap_or(32.0);

    Some((columns * cell_width, rows * cell_height))
}

fn estimated_text_size(component: &Mapping) -> Option<(f32, f32)> {
    let content = string_field(component, "content")?;
    Some(((content.chars().count() as f32).max(1.0) * 18.0, 36.0))
}

fn object_category(component_types: &[String], has_transform3: bool) -> String {
    let joined = component_types.join(" ").to_lowercase();
    if has_transform3
        || joined.contains("3d")
        || joined.contains("mesh")
        || joined.contains("material")
    {
        return "3d".to_owned();
    }
    if joined.contains("camera") {
        return "camera".to_owned();
    }
    if joined.contains("ui") || joined.contains("button") {
        return "ui".to_owned();
    }
    if joined.contains("sprite") || joined.contains("text") || joined.contains("vector") {
        return "render".to_owned();
    }
    if joined.contains("tile") {
        return "tilemap".to_owned();
    }
    if joined.contains("physics") || joined.contains("collider") || joined.contains("body") {
        return "physics".to_owned();
    }
    if joined.contains("light") {
        return "light".to_owned();
    }
    if joined.contains("script") || joined.contains("behavior") {
        return "script".to_owned();
    }
    "other".to_owned()
}

fn infer_canvas_kind_from_objects(objects: &[EditorSceneObjectDto]) -> EditorSceneCanvasKindDto {
    if objects
        .iter()
        .any(|object| object.transform_3.is_some() || object.category == "3d")
    {
        return EditorSceneCanvasKindDto::ThreeD;
    }

    EditorSceneCanvasKindDto::TwoD
}

fn append_object_quality_diagnostics(
    objects: &[EditorSceneObjectDto],
    diagnostics: &mut Vec<EditorDiagnosticDto>,
) {
    for object in objects {
        if object.transform_2.is_none() && object.transform_3.is_none() {
            push_diagnostic(
                diagnostics,
                DiagnosticLevel::Info,
                DIAG_ENTITY_NO_TRANSFORM2,
                format!(
                    "Entity `{}` has no transform2/transform3 and is not editable in the viewport.",
                    object.entity_id
                ),
            );
            continue;
        }

        if object.transform_2.is_some() && object.bounds_2.is_none() {
            let code = if object.category == "script" || object.category == "other" {
                DIAG_COMPONENT_BOUNDS_UNSUPPORTED
            } else {
                DIAG_ENTITY_NO_BOUNDS2
            };
            push_diagnostic(
                diagnostics,
                DiagnosticLevel::Info,
                code,
                format!(
                    "Entity `{}` has transform2 but no supported 2D bounds provider.",
                    object.entity_id
                ),
            );
        }
    }
}

fn snapshot_quality(
    indexed_entities: usize,
    objects: &[EditorSceneObjectDto],
    diagnostics: &[EditorDiagnosticDto],
) -> EditorSceneSnapshotQualityDto {
    let mut diagnostics_by_code = BTreeMap::new();
    for diagnostic in diagnostics {
        *diagnostics_by_code
            .entry(diagnostic.code.clone())
            .or_insert(0) += 1;
    }

    EditorSceneSnapshotQualityDto {
        indexed_entities,
        objects: objects.len(),
        editable_objects: objects
            .iter()
            .filter(|object| object.selectable && !object.locked && object.bounds_2.is_some())
            .count(),
        objects_without_transform: objects
            .iter()
            .filter(|object| object.transform_2.is_none() && object.transform_3.is_none())
            .count(),
        objects_without_bounds: objects
            .iter()
            .filter(|object| object.bounds_2.is_none())
            .count(),
        unsupported_bounds_providers: diagnostics_by_code
            .get(DIAG_COMPONENT_BOUNDS_UNSUPPORTED)
            .copied()
            .unwrap_or(0),
        diagnostics_by_code,
    }
}

fn push_diagnostic(
    diagnostics: &mut Vec<EditorDiagnosticDto>,
    level: DiagnosticLevel,
    code: &str,
    message: impl Into<String>,
) {
    diagnostics.push(EditorDiagnosticDto {
        level,
        code: code.to_owned(),
        message: message.into(),
        path: None,
    });
}

fn mapping(value: &Value) -> Option<&Mapping> {
    value.as_mapping()
}

fn string_field(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn bool_field(mapping: &Mapping, key: &str) -> Option<bool> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_bool)
}

fn number_field(mapping: &Mapping, key: &str) -> Option<f32> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(number_value)
}

fn number_value(value: &Value) -> Option<f32> {
    value
        .as_f64()
        .map(|value| value as f32)
        .or_else(|| value.as_i64().map(|value| value as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_snapshot_extracts_sprite_and_text_bounds() {
        let yaml = r#"
version: 1
scene:
  id: hello
  label: Hello
entities:
  - id: square
    name: square
    transform2:
      translation: { x: 10.0, y: 20.0 }
      rotation_radians: 0.5
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Sprite2D
        size: { x: 100.0, y: 50.0 }
  - id: label
    name: label
    transform2:
      translation: { x: 0.0, y: -100.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Text2D
        content: HELLO
        bounds: { x: 200.0, y: 40.0 }
"#;

        let value = serde_yaml::from_str::<Value>(yaml).unwrap();
        let snapshot =
            snapshot_from_scene_value("mod".to_owned(), "hello".to_owned(), &value).unwrap();

        assert!(matches!(
            snapshot.layout_source,
            EditorSceneSnapshotLayoutSourceDto::Document
        ));
        assert_eq!(snapshot.objects.len(), 2);
        assert_eq!(snapshot.objects[0].bounds_2.as_ref().unwrap().width, 100.0);
        assert_eq!(snapshot.objects[1].bounds_2.as_ref().unwrap().height, 40.0);
    }

    #[test]
    fn document_snapshot_does_not_create_fake_bounds_for_script_only_entity() {
        let yaml = r#"
version: 1
scene: { id: script-only }
entities:
  - id: controller
    name: controller
    components:
      - type: Behavior
        kind: scene_controller
"#;

        let value = serde_yaml::from_str::<Value>(yaml).unwrap();
        let snapshot =
            snapshot_from_scene_value("mod".to_owned(), "script-only".to_owned(), &value).unwrap();

        assert_eq!(snapshot.objects.len(), 1);
        assert!(snapshot.objects[0].bounds_2.is_none());
        assert!(snapshot.objects[0].transform_2.is_none());
        assert_eq!(snapshot.quality.editable_objects, 0);
        assert_eq!(
            snapshot
                .quality
                .diagnostics_by_code
                .get(DIAG_ENTITY_NO_TRANSFORM2),
            Some(&1)
        );
    }

    #[test]
    fn document_snapshot_reports_quality_for_mixed_scene() {
        let yaml = r#"
version: 1
scene:
  id: mixed
entities:
  - id: sprite
    name: sprite
    transform2:
      translation: { x: 10.0, y: 20.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Sprite2D
        size: { x: 64.0, y: 32.0 }
  - id: logic
    name: logic
    transform2:
      translation: { x: 0.0, y: 0.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Behavior
        kind: controller
  - id: script-only
    name: script-only
    components:
      - type: Behavior
        kind: scene_controller
"#;

        let value = serde_yaml::from_str::<Value>(yaml).unwrap();
        let snapshot =
            snapshot_from_scene_value("mod".to_owned(), "mixed".to_owned(), &value).unwrap();

        assert_eq!(snapshot.quality.indexed_entities, 3);
        assert_eq!(snapshot.quality.objects, 3);
        assert_eq!(snapshot.quality.editable_objects, 2);
        assert_eq!(snapshot.quality.objects_without_transform, 1);
        assert_eq!(snapshot.quality.objects_without_bounds, 1);
        assert_eq!(
            snapshot
                .quality
                .diagnostics_by_code
                .get(DIAG_ENTITY_NO_TRANSFORM2),
            Some(&1)
        );
    }
}
