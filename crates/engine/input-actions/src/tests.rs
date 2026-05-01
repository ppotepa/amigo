mod tests {
    use super::*;

    #[test]
    fn action_axis_uses_active_map() {
        let input = InputState::default();
        let actions = InputActionService::default();
        actions.register_map(
            InputActionMap {
                id: "gameplay".to_owned(),
                actions: BTreeMap::from([(
                    InputActionId::new("actor.move_forward"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::W],
                        negative: vec![KeyCode::S],
                    },
                )]),
            },
            true,
        );

        input.set_key(KeyCode::W, true);
        assert_eq!(actions.axis(&input, "actor.move_forward"), 1.0);
        input.set_key(KeyCode::W, false);
        input.set_key(KeyCode::S, true);
        assert_eq!(actions.axis(&input, "actor.move_forward"), -1.0);
    }

    #[test]
    fn action_button_tracks_pressed_and_down() {
        let input = InputState::default();
        let actions = InputActionService::default();
        actions.register_map(
            InputActionMap {
                id: "gameplay".to_owned(),
                actions: BTreeMap::from([(
                    InputActionId::new("actor.primary_action"),
                    InputActionBinding::Button {
                        pressed: vec![KeyCode::Space],
                    },
                )]),
            },
            true,
        );

        input.set_key(KeyCode::Space, true);
        assert!(actions.down(&input, "actor.primary_action"));
        assert!(actions.pressed(&input, "actor.primary_action"));
        input.clear_frame_transients();
        assert!(actions.down(&input, "actor.primary_action"));
        assert!(!actions.pressed(&input, "actor.primary_action"));
    }

    #[test]
    fn inactive_or_missing_action_is_safe() {
        let input = InputState::default();
        let actions = InputActionService::default();

        assert_eq!(actions.axis(&input, "missing"), 0.0);
        assert!(!actions.down(&input, "missing"));
        assert!(!actions.pressed(&input, "missing"));
    }
}
