    use std::path::PathBuf;

    use super::{
        SceneComponentDocument, SceneEntitySelectorDocument, SceneEntitySelectorKindDocument,
        load_scene_document_from_path, load_scene_document_from_str,
    };

    #[test]
    fn parses_scene_document_from_yaml() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        assert_eq!(document.scene.id, "sprite-lab");
        assert_eq!(document.entities.len(), 2);
        assert_eq!(document.entity_names()[1], "playground-2d-sprite");
        assert_eq!(
            document.component_kind_counts().get("Sprite2D"),
            Some(&1usize)
        );
        assert!(matches!(
            document.entities[1].components[0],
            SceneComponentDocument::Sprite2d { .. }
        ));
    }

    #[test]
    fn parses_entity_lifecycle_groups_and_properties() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy, flying]
    groups: [wave-1]
    visible: false
    simulation_enabled: true
    collision_enabled: false
    properties:
      score_value: 100
      speed: 2.5
      elite: true
      label: scout
"#,
        )
        .expect("scene document should parse");

        let entity = &document.entities[0];
        assert_eq!(entity.tags, vec!["enemy".to_owned(), "flying".to_owned()]);
        assert_eq!(entity.groups, vec!["wave-1".to_owned()]);
        assert!(!entity.visible);
        assert!(entity.simulation_enabled);
        assert!(!entity.collision_enabled);
        assert!(entity.properties.contains_key("score_value"));
        assert!(entity.properties.contains_key("speed"));
        assert!(entity.properties.contains_key("elite"));
        assert!(entity.properties.contains_key("label"));
    }

    #[test]
    fn parses_entity_selector_documents_from_yaml() {
        let selectors = serde_yaml::from_str::<Vec<SceneEntitySelectorDocument>>(
            r#"
- kind: entity
  value: player
- kind: tag
  value: enemy
- kind: group
  value: wave-1
- kind: pool
  value: bullets
"#,
        )
        .expect("selector documents should parse");

        assert_eq!(
            selectors,
            vec![
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Entity,
                    value: "player".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Tag,
                    value: "enemy".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Group,
                    value: "wave-1".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Pool,
                    value: "bullets".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_collision_event_rules_from_yaml() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: collision-preview
collision_events:
  - id: projectile-hits-target
    source:
      kind: tag
      value: projectile
    target:
      kind: group
      value: targets
    event: collision.hit
entities: []
"#,
        )
        .expect("scene document should parse");

        assert_eq!(document.collision_events.len(), 1);
        assert_eq!(document.collision_events[0].id, "projectile-hits-target");
        assert_eq!(
            document.collision_events[0].source,
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Tag,
                value: "projectile".to_owned(),
            }
        );
        assert!(document.collision_events[0].once_per_overlap);
    }

    #[test]
    fn parses_playground_scene_documents_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let sprite_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/sprite-lab/scene.yml"),
        )
        .expect("sprite lab scene should parse");
        let material_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
        )
        .expect("material lab scene should parse");

        assert_eq!(sprite_doc.scene.id, "sprite-lab");
        assert_eq!(material_doc.scene.id, "material-lab");
        assert!(sprite_doc.component_kind_counts().contains_key("Sprite2D"));
        assert!(
            material_doc
                .component_kind_counts()
                .contains_key("Material3D")
        );
    }

    #[test]
    fn parses_playground_2d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/hello-world-spritesheet/scene.yml"),
        )
        .expect("playground 2d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-spritesheet");
        assert_eq!(document.transitions.len(), 1);
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(document.component_kind_counts().contains_key("Text2D"));
    }

    #[test]
    fn parses_playground_3d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/hello-world-cube/scene.yml"),
        )
        .expect("playground 3d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-cube");
        assert!(document.component_kind_counts().contains_key("Mesh3D"));
        assert!(document.component_kind_counts().contains_key("Material3D"));
        assert!(document.component_kind_counts().contains_key("Text3D"));
    }

    #[test]
    fn parses_playground_2d_screen_space_preview_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/screen-space-preview/scene.yml"),
        )
        .expect("screen-space preview scene should parse");

        assert_eq!(document.scene.id, "screen-space-preview");
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(document.component_kind_counts().contains_key("UiDocument"));
    }

    #[test]
    fn parses_sidescroller_component_document_from_yaml() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: vertical-slice
  label: Vertical Slice
entities:
  - id: camera
    name: playground-sidescroller-camera
    components:
      - type: Camera2D
      - type: CameraFollow2D
        target: playground-sidescroller-player
  - id: tilemap
    name: playground-sidescroller-tilemap
    components:
      - type: TileMap2D
        tileset: playground-sidescroller/tilesets/platformer
        ruleset: playground-sidescroller/tilesets/platformer-rules
        tile_size: { x: 16.0, y: 16.0 }
        grid:
          - "...."
          - ".P.."
          - "####"
  - id: player
    name: playground-sidescroller-player
    components:
      - type: TileMapMarker2D
        tilemap_entity: playground-sidescroller-tilemap
        symbol: "P"
        offset: { x: 0.0, y: 8.0 }
      - type: KinematicBody2D
        velocity: { x: 0.0, y: 0.0 }
        gravity_scale: 1.0
        terminal_velocity: 720.0
      - type: AabbCollider2D
        size: { x: 20.0, y: 30.0 }
        offset: { x: 0.0, y: 1.0 }
        layer: player
        mask: [world, trigger]
      - type: MotionController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
  - id: coin
    name: playground-sidescroller-coin
    components:
      - type: Sprite2D
        texture: playground-sidescroller/textures/coin
        size: { x: 16.0, y: 16.0 }
        animation:
          fps: 10.0
          looping: true
      - type: Trigger2D
        size: { x: 16.0, y: 16.0 }
        layer: trigger
        mask: [player]
        event: coin.collected
"#####,
        )
        .expect("sidescroller scene document should parse");

        assert_eq!(document.scene.id, "vertical-slice");
        assert!(document.component_kind_counts().contains_key("TileMap2D"));
        let tilemap_component = document
            .entities
            .iter()
            .find(|entity| entity.name == "playground-sidescroller-tilemap")
            .and_then(|entity| {
                entity
                    .components
                    .iter()
                    .find(|component| matches!(component, SceneComponentDocument::TileMap2d { .. }))
            })
            .expect("tilemap component should exist");
        match tilemap_component {
            SceneComponentDocument::TileMap2d { ruleset, .. } => {
                assert_eq!(
                    ruleset.as_deref(),
                    Some("playground-sidescroller/tilesets/platformer-rules")
                );
            }
            _ => unreachable!("expected tilemap component"),
        }
        assert!(
            document
                .component_kind_counts()
                .contains_key("KinematicBody2D")
        );
        assert!(
            document
                .component_kind_counts()
                .contains_key("AabbCollider2D")
        );
        assert!(document.component_kind_counts().contains_key("Trigger2D"));
        assert!(
            document
                .component_kind_counts()
                .contains_key("MotionController2D")
        );
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(
            document
                .component_kind_counts()
                .contains_key("CameraFollow2D")
        );
        assert!(
            document
                .component_kind_counts()
                .contains_key("TileMapMarker2D")
        );
    }

    #[test]
    fn rejects_legacy_platformer_controller_component_alias() {
        let result = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: legacy-motion-alias
  label: Legacy Motion Alias
entities:
  - id: player
    components:
      - type: PlatformerController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
"#,
        );

        assert!(result.is_err());
    }
