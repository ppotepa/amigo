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
    let snapshot = snapshot_from_scene_value("mod".to_owned(), "hello".to_owned(), &value).unwrap();

    assert!(matches!(
        snapshot.layout_source,
        EditorSceneSnapshotLayoutSourceDto::Document
    ));
    assert_eq!(snapshot.objects.len(), 2);
    assert_eq!(snapshot.objects[0].bounds_2.as_ref().unwrap().width, 100.0);
    assert_eq!(snapshot.objects[1].bounds_2.as_ref().unwrap().height, 40.0);
}

#[test]
fn document_snapshot_uses_camera2d_transform() {
    let yaml = r#"
version: 1
scene: { id: camera-scene }
entities:
  - id: menu-camera
    transform2:
      translation: { x: -68.0, y: 49.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Camera2D
  - id: title
    transform2:
      translation: { x: -304.0, y: 178.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Text2D
        content: INK WARS
        bounds: { x: 560.0, y: 96.0 }
"#;

    let value = serde_yaml::from_str::<Value>(yaml).unwrap();
    let snapshot =
        snapshot_from_scene_value("mod".to_owned(), "camera-scene".to_owned(), &value).unwrap();

    assert_eq!(snapshot.camera.x, -68.0);
    assert_eq!(snapshot.camera.y, 49.0);
    assert_eq!(snapshot.camera.zoom, 1.0);
}

#[test]
fn text_bounds_match_current_renderer_metrics() {
    let yaml = r#"
version: 1
scene: { id: text-scene }
entities:
  - id: title
    transform2:
      translation: { x: -304.0, y: 178.0 }
      scale: { x: 1.0, y: 1.0 }
    components:
      - type: Text2D
        content: INK WARS
        bounds: { x: 560.0, y: 96.0 }
"#;

    let value = serde_yaml::from_str::<Value>(yaml).unwrap();
    let snapshot =
        snapshot_from_scene_value("mod".to_owned(), "text-scene".to_owned(), &value).unwrap();
    let bounds = snapshot.objects[0].bounds_2.as_ref().unwrap();

    assert!((bounds.width - 658.2857).abs() < 0.01);
    assert_eq!(bounds.height, 96.0);
    assert!((bounds.x - -633.1428).abs() < 0.01);
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
    let snapshot = snapshot_from_scene_value("mod".to_owned(), "mixed".to_owned(), &value).unwrap();

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
    assert_eq!(
        snapshot
            .quality
            .diagnostics_by_code
            .get(DIAG_ENTITY_LOCKED_BY_PLACEMENT),
        None
    );
}

#[test]
fn document_snapshot_detects_tilemap_marker_placement() {
    let yaml = r#"
version: 1
scene: { id: marker-scene }
entities:
  - id: player
    components:
      - type: TileMapMarker2D
        offset: { x: 12.0, y: 24.0 }
      - type: Sprite2D
        size: { x: 32.0, y: 32.0 }
"#;

    let value = serde_yaml::from_str::<Value>(yaml).unwrap();
    let snapshot =
        snapshot_from_scene_value("mod".to_owned(), "marker-scene".to_owned(), &value).unwrap();
    let object = &snapshot.objects[0];

    assert_eq!(
        object.placement_kind,
        EditorObjectPlacementKindDto::TilemapMarker
    );
    assert_eq!(
        object.edit_command_kind,
        EditorObjectEditCommandKindDto::SetTilemapMarkerOffset
    );
    assert!(object.movable);
    assert_eq!(object.transform_2.as_ref().map(|value| value.x), Some(12.0));
    assert!(object.selection_bounds_2.is_some());
}

#[test]
fn document_snapshot_detects_attached_placement() {
    let yaml = r#"
version: 1
scene: { id: attached-scene }
entities:
  - id: player-indicator
    components:
      - type: ParticleEmitter2D
        attached_to: player
        local_offset: { x: 6.0, y: -4.0 }
"#;

    let value = serde_yaml::from_str::<Value>(yaml).unwrap();
    let snapshot =
        snapshot_from_scene_value("mod".to_owned(), "attached-scene".to_owned(), &value).unwrap();
    let object = &snapshot.objects[0];

    assert_eq!(
        object.placement_kind,
        EditorObjectPlacementKindDto::Attached
    );
    assert_eq!(
        object.edit_command_kind,
        EditorObjectEditCommandKindDto::SetAttachedLocalOffset
    );
    assert!(object.movable);
    assert_eq!(object.transform_2.as_ref().map(|value| value.y), Some(-4.0));
    assert!(object.selection_bounds_2.is_some());
}
