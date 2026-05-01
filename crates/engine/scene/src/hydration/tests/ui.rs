    use std::path::PathBuf;

    

    use super::super::build_scene_hydration_plan;
    use crate::{
        SceneCommand,
        load_scene_document_from_path, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_ui_theme_set_and_style_class() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-theme-test
  label: UI Theme Test
entities:
  - id: ui
    name: playground-hud-ui
    components:
      - type: UiThemeSet
        active: space_dark
        themes:
          - id: space_dark
            palette:
              background: "#050812FF"
              surface: "#101827DD"
              surface_alt: "#172033DD"
              text: "#EAF6FFFF"
              text_muted: "#89A2B7FF"
              border: "#2A6F9EFF"
              accent: "#39D7FFFF"
              accent_text: "#001018FF"
              danger: "#FF4D6DFF"
              warning: "#FFB000FF"
              success: "#5CFF9CFF"
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: column
          id: root
          style_class: root
          children:
            - type: text
              id: title
              text: THEMED
              style_class: text_title
"#####,
        )
        .expect("ui theme scene should parse");
        let plan = build_scene_hydration_plan("playground-hud-ui", &document)
            .expect("ui theme plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUiThemeSet { command }
                if command.entity_name == "playground-hud-ui"
                    && command.active.as_deref() == Some("space_dark")
                    && command.themes.len() == 1
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.document.root.style_class.as_deref() == Some("root")
                    && command.document.root.children[0].style_class.as_deref() == Some("text_title")
        )));
    }

    #[test]
    fn hydrates_native_tab_view_and_group_box() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-native-controls
entities:
  - id: ui
    name: ui
    components:
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: tab-view
          id: tabs
          selected: settings
          tabs:
            - id: overview
              label: Overview
            - id: settings
              label: Settings
          on_change:
            event: ui.tab.changed
          children:
            - type: group-box
              id: overview
              text: Overview Group
            - type: group-box
              id: settings
              text: Settings Group
"#####,
        )
        .expect("native ui control scene should parse");
        let plan = build_scene_hydration_plan("test", &document)
            .expect("native ui control scene should hydrate");

        let ui = plan
            .commands
            .iter()
            .find_map(|command| match command {
                SceneCommand::QueueUi { command } => Some(command),
                _ => None,
            })
            .expect("ui command should be queued");
        match &ui.document.root.kind {
            crate::SceneUiNodeKind::TabView { selected, tabs, .. } => {
                assert_eq!(selected, "settings");
                assert_eq!(tabs.len(), 2);
            }
            _ => panic!("expected tab view root"),
        }
        assert!(matches!(
            ui.document.root.children[0].kind,
            crate::SceneUiNodeKind::GroupBox { .. }
        ));
    }

    #[test]
    fn hydrates_curve_editor_points_and_legacy_values() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-curve-editor
entities:
  - id: ui
    name: ui
    components:
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: column
          id: root
          children:
            - type: curve-editor
              id: curve
              points:
                - { t: -1.0, value: 0.25 }
                - { t: 0.5, value: 2.0 }
              on_change:
                event: ui.curve.changed
"#####,
        )
        .expect("curve editor scene should parse");
        let plan = build_scene_hydration_plan("test", &document)
            .expect("curve editor scene should hydrate");
        let ui = plan
            .commands
            .iter()
            .find_map(|command| match command {
                SceneCommand::QueueUi { command } => Some(command),
                _ => None,
            })
            .expect("ui command should be queued");
        let curve = &ui.document.root.children[0];

        match &curve.kind {
            crate::SceneUiNodeKind::CurveEditor { points } => {
                assert_eq!(points.len(), 4);
                assert_eq!(points[0].t, 0.0);
                assert_eq!(points[1].value, 1.0);
            }
            _ => panic!("expected curve editor"),
        }
        assert_eq!(
            curve
                .on_change
                .as_ref()
                .map(|binding| binding.event.as_str()),
            Some("ui.curve.changed")
        );
    }

    #[test]
    fn builds_hydration_plan_for_hud_ui_showcase() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-hud-ui/scenes/showcase/scene.yml"),
        )
        .expect("hud ui showcase scene should parse");
        let plan = build_scene_hydration_plan("playground-hud-ui", &document)
            .expect("hud ui showcase plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUiThemeSet { command }
                if command.active.as_deref() == Some("space_dark") && command.themes.len() == 2
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.entity_name == "playground-hud-ui-showcase"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_vector_arcade_scene() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: vector-arcade
entities:
  - id: actor
    name: test-actor
    components:
      - type: VectorShape2D
        kind: polygon
        points:
          - { x: 0.0, y: -6.0 }
          - { x: 12.0, y: 0.0 }
          - { x: 0.0, y: 6.0 }
        stroke_color: "#FFFFFFFF"
  - id: target-pool
    name: test-target-pool
    components:
      - type: EntityPool
        pool: targets
        members:
          - test-target-a
"#####,
        )
        .expect("vector arcade scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("vector arcade scene plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueVectorShape2d { command }
                if command.entity_name == "test-actor"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueEntityPool { command } if command.pool == "targets"
        )));
    }

