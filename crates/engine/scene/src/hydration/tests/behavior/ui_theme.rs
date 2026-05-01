    

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_generic_ui_theme_switcher_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: theme-switcher
    name: theme-switcher
    components:
      - type: Behavior
        kind: ui_theme_switcher
        bindings:
          ui.theme.space_dark: space_dark
          ui.theme.clean_dev: clean_dev
        cycle: ui.theme.cycle
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "theme-switcher"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::UiThemeSwitcher {
                            bindings,
                            cycle_action
                        } if bindings.get("ui.theme.space_dark").map(String::as_str) == Some("space_dark")
                            && cycle_action.as_deref() == Some("ui.theme.cycle")
                    )
        )));
    }

