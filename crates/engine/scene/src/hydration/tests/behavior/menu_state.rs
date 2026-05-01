    

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_menu_navigation_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: menu-controller
    name: menu-controller
    components:
      - type: Behavior
        kind: menu_navigation_controller
        index_state: menu_index
        item_count: 4
        item_count_state: menu_count
        up_action: menu.up
        down_action: menu.down
        confirm_action: menu.confirm
        move_audio: menu-move
        confirm_audio: menu-select
        selected_color_prefix: menu_color
        selected_color: "#FFFFFFFF"
        unselected_color: "#9A9A9AFF"
        confirm_events:
          - menu.start
          - menu.options
          - menu.highscores
          - menu.quit
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "menu-controller"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::MenuNavigationController {
                            index_state,
                            item_count,
                            item_count_state,
                            up_action,
                            down_action,
                            confirm_action,
                            move_audio,
                            confirm_audio,
                            confirm_events,
                            selected_color_prefix,
                            selected_color,
                            unselected_color,
                            wrap,
                        } if index_state == "menu_index"
                            && *item_count == 4
                            && item_count_state.as_deref() == Some("menu_count")
                            && up_action == "menu.up"
                            && down_action == "menu.down"
                            && confirm_action.as_deref() == Some("menu.confirm")
                            && move_audio.as_deref() == Some("menu-move")
                            && confirm_audio.as_deref() == Some("menu-select")
                            && confirm_events.len() == 4
                            && selected_color_prefix.as_deref() == Some("menu_color")
                            && selected_color == "#FFFFFFFF"
                            && unselected_color == "#9A9A9AFF"
                            && *wrap
                    )
        )));
    }

    #[test]
    fn hydrates_state_action_controller_behavior_commands() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: state-controllers
    name: state-controllers
    components:
      - type: Behavior
        enabled_when:
          state: game_mode
          not_equals: playing
        kind: set_state_on_action_controller
        action: ui.open
        key: panel
        value: settings
        audio: click
      - type: Behavior
        kind: toggle_state_controller
        action: debug.toggle
        key: debug_visible
        default: false
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "state-controllers"
                    && command
                        .condition
                        .as_ref()
                        .is_some_and(|condition| condition.state_key == "game_mode"
                            && condition.not_equals.as_deref() == Some("playing"))
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::SetStateOnActionController {
                            action,
                            key,
                            value,
                            audio,
                        } if action == "ui.open"
                            && key == "panel"
                            && value == "settings"
                            && audio.as_deref() == Some("click")
                    )
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "state-controllers"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::ToggleStateController {
                            action,
                            key,
                            default,
                            audio,
                        } if action == "debug.toggle"
                            && key == "debug_visible"
                            && !*default
                            && audio.is_none()
                    )
        )));
    }

    #[test]
    fn hydrates_behavior_condition_numeric_and_bool_checks() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: conditional-controller
    name: conditional-controller
    components:
      - type: Behavior
        enabled_when:
          state: charge
          greater_or_equal: 0.5
          less_than: 1.0
        kind: set_state_on_action_controller
        action: ui.open
        key: panel
        value: settings
      - type: Behavior
        enabled_when:
          state: debug_visible
          is_true: true
        kind: toggle_state_controller
        action: debug.toggle
        key: debug_visible
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "conditional-controller"
                    && command
                        .condition
                        .as_ref()
                        .is_some_and(|condition| condition.state_key == "charge"
                            && condition.greater_or_equal == Some(0.5)
                            && condition.less_than == Some(1.0))
        )));

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "conditional-controller"
                    && command
                        .condition
                        .as_ref()
                        .is_some_and(|condition| condition.state_key == "debug_visible"
                            && condition.is_true)
        )));
    }

