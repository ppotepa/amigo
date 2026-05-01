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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorCondition {
    pub state_key: String,
    pub equals: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorKind {
    FreeflightInputController(FreeflightInputControllerBehavior),
    ParticleIntensityController(ParticleIntensityControllerBehavior),
    ProjectileFireController(ProjectileFireControllerBehavior),
    MenuNavigationController(MenuNavigationControllerBehavior),
    SceneTransitionController(SceneTransitionControllerBehavior),
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
pub struct MenuNavigationControllerBehavior {
    pub index_state: String,
    pub item_count: i64,
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
                equals: "showcase".to_owned(),
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
    fn behavior_service_accepts_menu_navigation_controller() {
        let service = BehaviorSceneService::default();
        service.queue(BehaviorCommand {
            source_mod: "test".to_owned(),
            entity_name: "menu-nav".to_owned(),
            condition: None,
            behavior: BehaviorKind::MenuNavigationController(MenuNavigationControllerBehavior {
                index_state: "menu_index".to_owned(),
                item_count: 3,
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
