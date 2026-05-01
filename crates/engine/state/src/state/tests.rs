mod tests {
    use super::{SceneStateService, SceneTimerService, SessionStateService};

    #[test]
    fn scene_state_set_get_add_and_clear() {
        let state = SceneStateService::default();

        assert!(state.set_int("score", 10));
        assert!(state.set_float("speed", 1.5));
        assert!(state.set_bool("armed", false));
        assert!(state.set_string("label", "wave"));

        assert_eq!(state.get_int("score"), Some(10));
        assert_eq!(state.add_int("score", 5), 15);
        assert_eq!(state.add_float("speed", 0.25), 1.75);
        assert!(state.add_bool("armed", true));
        assert_eq!(state.add_string("label", " 1"), "wave 1");

        state.clear_scene();

        assert_eq!(state.get_int("score"), None);
        assert_eq!(state.get_string("label"), None);
    }

    #[test]
    fn scene_timers_start_tick_ready_after_and_reset() {
        let timers = SceneTimerService::default();

        assert!(timers.start("cooldown", 0.5));
        assert!(timers.active("cooldown"));
        assert!(!timers.ready("cooldown"));

        timers.tick(0.25);
        assert!(timers.active("cooldown"));

        timers.tick(0.25);
        assert!(!timers.active("cooldown"));
        assert!(timers.ready("cooldown"));

        assert!(!timers.after("spawn", 0.25));
        timers.tick(0.25);
        assert!(timers.after("spawn", 0.25));
        assert!(!timers.active("spawn"));

        timers.reset_scene();
        assert!(!timers.ready("cooldown"));
    }

    #[test]
    fn session_state_survives_scene_state_clear() {
        let scene = SceneStateService::default();
        let session = SessionStateService::default();

        assert!(scene.set_int("score", 120));
        assert!(session.set_bool("game.option", true));
        assert!(session.set_int("game.highscore.1", 10_000));

        scene.clear_scene();

        assert_eq!(scene.get_int("score"), None);
        assert_eq!(session.get_bool("game.option"), Some(true));
        assert_eq!(session.add_int("game.highscore.1", 250), 10_250);
    }
}
