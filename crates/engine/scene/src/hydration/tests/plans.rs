    use std::path::PathBuf;

    use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};

    use super::super::{
        build_scene_hydration_plan, entity_selector_from_document, scene_key_from_document,
    };
    use crate::{
        EntitySelector, SceneCommand, SceneEntitySelectorDocument,
        SceneEntitySelectorKindDocument,
        load_scene_document_from_path, load_scene_document_from_str,
    };

    #[test]
    fn builds_hydration_plan_for_2d_scene_document() {
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
      rotation_radians: 0.5
      scale: { x: 2.0, y: 3.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert_eq!(scene_key_from_document(&document).as_str(), "sprite-lab");
        assert_eq!(plan.commands.len(), 5);
        assert!(matches!(
            &plan.commands[0],
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { .. })
            } if name == "playground-2d-camera"
        ));
        assert!(matches!(
            &plan.commands[4],
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-sprite"
                    && command.size == Vec2::new(128.0, 128.0)
                    && command.transform == Transform2 {
                        translation: Vec2::new(12.0, -4.0),
                        rotation_radians: 0.5,
                        scale: Vec2::new(2.0, 3.0),
                    }
        ));
    }

    #[test]
    fn builds_hydration_plan_for_entity_metadata() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy]
    groups: [wave-1]
    visible: false
    collision_enabled: false
    properties:
      score_value: 100
      label: scout
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("metadata-preview", &document).expect("plan should build");

        assert!(matches!(
            &plan.commands[1],
            SceneCommand::ConfigureEntity {
                entity_name,
                lifecycle,
                tags,
                groups,
                properties,
            } if entity_name == "actor"
                && !lifecycle.visible
                && lifecycle.simulation_enabled
                && !lifecycle.collision_enabled
                && tags == &vec!["enemy".to_owned()]
                && groups == &vec!["wave-1".to_owned()]
                && properties.contains_key("score_value")
                && properties.contains_key("label")
        ));
    }

    #[test]
    fn converts_selector_documents_to_runtime_selectors() {
        let cases = [
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Entity,
                    value: "player".to_owned(),
                },
                EntitySelector::Entity("player".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Tag,
                    value: "enemy".to_owned(),
                },
                EntitySelector::Tag("enemy".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Group,
                    value: "wave-1".to_owned(),
                },
                EntitySelector::Group("wave-1".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Pool,
                    value: "bullets".to_owned(),
                },
                EntitySelector::Pool("bullets".to_owned()),
            ),
        ];

        for (document, expected) in cases {
            assert_eq!(entity_selector_from_document(&document), expected);
            assert_eq!(EntitySelector::from(document), expected);
        }
    }

    #[test]
    fn builds_hydration_plan_for_collision_event_rules() {
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
    once_per_overlap: true
entities: []
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("collision-preview", &document).expect("plan should build");

        assert!(matches!(
            &plan.commands[0],
            SceneCommand::QueueCollisionEventRule2d { command }
                if command.id == "projectile-hits-target"
                    && command.source == EntitySelector::Tag("projectile".to_owned())
                    && command.target == EntitySelector::Group("targets".to_owned())
                    && command.event == "collision.hit"
                    && command.once_per_overlap
        ));
    }

    #[test]
    fn builds_hydration_plan_for_material_scene_document() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
        )
        .expect("material lab scene should parse");

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { translation, scale, .. })
            } if name == "playground-3d-material-probe"
                && *translation == Vec3::ZERO
                && *scale == Vec3::ONE
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-material-probe"
                    && command.label == "debug-surface"
                    && command.albedo == ColorRgba::WHITE
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_2d_main_scene() {
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

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-spritesheet"
                    && command.sheet.as_ref().map(|sheet| sheet.frame_count) == Some(8)
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText2d { command }
                if command.entity_name == "playground-2d-hello"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_3d_main_scene() {
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

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMesh3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText3d { command }
                if command.entity_name == "playground-3d-hello"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_2d_screen_space_preview() {
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

        let plan = build_scene_hydration_plan("playground-2d", &document)
            .expect("screen-space preview plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-ui-preview-square"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.entity_name == "playground-2d-ui-preview"
        )));
    }

