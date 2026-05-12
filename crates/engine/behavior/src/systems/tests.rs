mod tests {
    use super::*;
    use amigo_behavior::BehaviorCondition;

    #[test]
    fn behavior_condition_matches_string_scene_state() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "playing");

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: Some("playing".to_owned()),
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_rejects_mismatched_scene_state() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "menu");

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: Some("playing".to_owned()),
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_not_equals() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "menu");

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: None,
                not_equals: Some("playing".to_owned()),
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_numeric_thresholds() {
        let scene_state = SceneStateService::default();
        scene_state.set_float("charge", 0.75);

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "charge".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: Some(0.5),
                greater_or_equal: None,
                less_than: None,
                less_or_equal: Some(1.0),
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "charge".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: Some(0.9),
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_bool_checks() {
        let scene_state = SceneStateService::default();
        scene_state.set_bool("debug_visible", true);

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "debug_visible".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: true,
                is_false: false,
            }),
            Some(&scene_state),
        ));

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "debug_visible".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: true,
            }),
            Some(&scene_state),
        ));
    }
}
