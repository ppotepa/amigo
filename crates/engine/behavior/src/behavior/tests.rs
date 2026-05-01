mod tests {
    use super::*;

    #[test]
    fn behavior_service_queues_and_clears_behaviors() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "controller".to_owned(),
            condition: None,
            behavior: BehaviorKind::ParticleIntensityController(
                ParticleIntensityControllerBehavior {
                    emitter: "test-emitter".to_owned(),
                    action: "actor.accelerate".to_owned(),
                },
            ),
        });

        assert_eq!(service.behaviors().len(), 1);
        service.clear();
        assert!(service.behaviors().is_empty());
    }

    #[test]
    fn behavior_service_accepts_ui_theme_switcher() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "theme-switcher".to_owned(),
            condition: Some(BehaviorCondition {
                state_key: "ui_mode".to_owned(),
                equals: Some("showcase".to_owned()),
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            behavior: BehaviorKind::UiThemeSwitcher(UiThemeSwitcherBehavior {
                bindings: BTreeMap::from([(
                    "ui.theme.space_dark".to_owned(),
                    "space_dark".to_owned(),
                )]),
                cycle_action: Some("ui.theme.cycle".to_owned()),
            }),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::UiThemeSwitcher(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_camera_follow_mode_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "camera-mode".to_owned(),
            condition: None,
            behavior: BehaviorKind::CameraFollowModeController(
                CameraFollowModeControllerBehavior {
                    camera: "camera".to_owned(),
                    action: "camera.fast".to_owned(),
                    target: Some("ship".to_owned()),
                    lerp: Some(0.12),
                    lookahead_velocity_scale: Some(0.35),
                    lookahead_max_distance: Some(180.0),
                    sway_amount: Some(18.0),
                    sway_frequency: Some(1.4),
                },
            ),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::CameraFollowModeController(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_projectile_fire_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "fire-controller".to_owned(),
            condition: None,
            behavior: BehaviorKind::ProjectileFireController(ProjectileFireControllerBehavior {
                emitter: "ship".to_owned(),
                source: None,
                action: "ship.fire".to_owned(),
                cooldown_seconds: 0.16,
                cooldown_id: Some("ship-fire".to_owned()),
                audio: Some("shot".to_owned()),
            }),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::ProjectileFireController(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_scene_transition_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "back-controller".to_owned(),
            condition: None,
            behavior: BehaviorKind::SceneTransitionController(SceneTransitionControllerBehavior {
                action: "ui.back".to_owned(),
                scene: "menu".to_owned(),
            }),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::SceneTransitionController(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_scene_auto_transition_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "alias".to_owned(),
            condition: None,
            behavior: BehaviorKind::SceneAutoTransitionController(
                SceneAutoTransitionControllerBehavior {
                    scene: "main-menu".to_owned(),
                },
            ),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::SceneAutoTransitionController(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_menu_navigation_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "menu-nav".to_owned(),
            condition: None,
            behavior: BehaviorKind::MenuNavigationController(MenuNavigationControllerBehavior {
                index_state: "menu_index".to_owned(),
                item_count: 3,
                item_count_state: Some("menu_count".to_owned()),
                up_action: "menu.up".to_owned(),
                down_action: "menu.down".to_owned(),
                confirm_action: Some("menu.confirm".to_owned()),
                wrap: true,
                move_audio: Some("menu-move".to_owned()),
                confirm_audio: Some("menu-select".to_owned()),
                confirm_events: vec!["start".to_owned(), "options".to_owned()],
                selected_color_prefix: Some("menu.color".to_owned()),
                selected_color: "#FFFFFFFF".to_owned(),
                unselected_color: "#999999FF".to_owned(),
            }),
        });

        assert!(matches!(
            service.behaviors().first().map(|command| &command.behavior),
            Some(BehaviorKind::MenuNavigationController(_))
        ));
    }

    #[test]
    fn behavior_service_accepts_state_action_controllers() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "set-state".to_owned(),
            condition: None,
            behavior: BehaviorKind::SetStateOnActionController(
                SetStateOnActionControllerBehavior {
                    action: "ui.open".to_owned(),
                    key: "panel".to_owned(),
                    value: "settings".to_owned(),
                    audio: None,
                },
            ),
        });
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "toggle-state".to_owned(),
            condition: None,
            behavior: BehaviorKind::ToggleStateController(ToggleStateControllerBehavior {
                action: "debug.toggle".to_owned(),
                key: "debug_visible".to_owned(),
                default: false,
                audio: Some("toggle".to_owned()),
            }),
        });

        assert!(service.behaviors().iter().any(|command| matches!(
            command.behavior,
            BehaviorKind::SetStateOnActionController(_)
        )));
        assert!(
            service
                .behaviors()
                .iter()
                .any(|command| matches!(command.behavior, BehaviorKind::ToggleStateController(_)))
        );
    }

    #[test]
    fn behavior_service_preserves_multiple_behaviors_on_same_entity() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "controls".to_owned(),
            condition: None,
            behavior: BehaviorKind::SceneTransitionController(SceneTransitionControllerBehavior {
                action: "ui.back".to_owned(),
                scene: "menu".to_owned(),
            }),
        });
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "controls".to_owned(),
            condition: None,
            behavior: BehaviorKind::ParticleIntensityController(
                ParticleIntensityControllerBehavior {
                    emitter: "test-emitter".to_owned(),
                    action: "actor.accelerate".to_owned(),
                },
            ),
        });

        assert_eq!(service.behaviors().len(), 2);
    }
}
