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
        self.behaviors
            .lock()
            .expect("behavior scene service mutex should not be poisoned")
            .insert(command.entity_name.clone(), command);
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
}
