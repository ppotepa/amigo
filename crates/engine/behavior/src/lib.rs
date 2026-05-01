use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub condition: Option<BehaviorCondition>,
    pub behavior: BehaviorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorCondition {
    pub state_key: String,
    pub equals: Option<String>,
    pub not_equals: Option<String>,
    pub greater_than: Option<f64>,
    pub greater_or_equal: Option<f64>,
    pub less_than: Option<f64>,
    pub less_or_equal: Option<f64>,
    pub is_true: bool,
    pub is_false: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorKind {
    FreeflightInputController(FreeflightInputControllerBehavior),
    CameraFollowModeController(CameraFollowModeControllerBehavior),
    ParticleIntensityController(ParticleIntensityControllerBehavior),
    ProjectileFireController(ProjectileFireControllerBehavior),
    MenuNavigationController(MenuNavigationControllerBehavior),
    SceneAutoTransitionController(SceneAutoTransitionControllerBehavior),
    SceneTransitionController(SceneTransitionControllerBehavior),
    SetStateOnActionController(SetStateOnActionControllerBehavior),
    ToggleStateController(ToggleStateControllerBehavior),
    UiThemeSwitcher(UiThemeSwitcherBehavior),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightInputControllerBehavior {
    pub target_entity: String,
    pub thrust_action: String,
    pub turn_action: String,
    pub strafe_action: Option<String>,
    pub thruster_emitter: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFollowModeControllerBehavior {
    pub camera: String,
    pub action: String,
    pub target: Option<String>,
    pub lerp: Option<f32>,
    pub lookahead_velocity_scale: Option<f32>,
    pub lookahead_max_distance: Option<f32>,
    pub sway_amount: Option<f32>,
    pub sway_frequency: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleIntensityControllerBehavior {
    pub emitter: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileFireControllerBehavior {
    pub emitter: String,
    pub source: Option<String>,
    pub action: String,
    pub cooldown_seconds: f32,
    pub cooldown_id: Option<String>,
    pub audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneTransitionControllerBehavior {
    pub action: String,
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneAutoTransitionControllerBehavior {
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuNavigationControllerBehavior {
    pub index_state: String,
    pub item_count: i64,
    pub item_count_state: Option<String>,
    pub up_action: String,
    pub down_action: String,
    pub confirm_action: Option<String>,
    pub wrap: bool,
    pub move_audio: Option<String>,
    pub confirm_audio: Option<String>,
    pub confirm_events: Vec<String>,
    pub selected_color_prefix: Option<String>,
    pub selected_color: String,
    pub unselected_color: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiThemeSwitcherBehavior {
    pub bindings: BTreeMap<String, String>,
    pub cycle_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetStateOnActionControllerBehavior {
    pub action: String,
    pub key: String,
    pub value: String,
    pub audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToggleStateControllerBehavior {
    pub action: String,
    pub key: String,
    pub default: bool,
    pub audio: Option<String>,
}

#[derive(Debug, Default)]
pub struct BehaviorSceneService {
    behaviors: Mutex<BTreeMap<String, BehaviorCommand>>,
}

impl BehaviorSceneService {
    pub fn queue(&self, command: BehaviorCommand) {
        let mut behaviors = self
            .behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned");
        let base_key = command.entity_name.clone();
        let mut key = base_key.clone();
        let mut suffix = 1;
        while behaviors.contains_key(&key) {
            suffix += 1;
            key = format!("{base_key}#{suffix}");
        }
        behaviors.insert(key, command);
    }

    pub fn behaviors(&self) -> Vec<BehaviorCommand> {
        self.behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned")
            .clear();
    }
}

pub struct BehaviorPlugin;

impl RuntimePlugin for BehaviorPlugin {
    fn name(&self) -> &'static str {
        "amigo-behavior"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(BehaviorSceneService::default())
    }
}

#[cfg(test)]
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
                    emitter: "thruster".to_owned(),
                    action: "ship.thrust".to_owned(),
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
                    emitter: "thruster".to_owned(),
                    action: "ship.thrust".to_owned(),
                },
            ),
        });

        assert_eq!(service.behaviors().len(), 2);
    }
}
