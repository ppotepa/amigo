    

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_scene_back_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: back-controller
    name: back-controller
    components:
      - type: Behavior
        kind: scene_back_controller
        action: ui.back
        scene: menu
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "back-controller"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::SceneTransitionController { action, scene }
                            if action == "ui.back" && scene == "menu"
                    )
        )));
    }

    #[test]
    fn hydrates_scene_auto_transition_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: alias-controller
    name: alias-controller
    components:
      - type: Behavior
        kind: scene_auto_transition_controller
        scene: main-menu
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "alias-controller"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::SceneAutoTransitionController { scene }
                            if scene == "main-menu"
                    )
        )));
    }

