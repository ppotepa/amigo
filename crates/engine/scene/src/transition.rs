use std::sync::Mutex;

use crate::{SceneCommand, SceneDocument, SceneDocumentError, SceneDocumentResult, SceneKey};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneTransitionPlan {
    pub source_mod: String,
    pub scene_id: String,
    pub transitions: Vec<SceneTransitionRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneTransitionRule {
    pub id: String,
    pub target_scene: SceneKey,
    pub trigger: SceneTransitionTrigger,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneTransitionTrigger {
    AfterSeconds { seconds: f32 },
    ScriptEvent { topic: String, payload: Vec<String> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneTransitionSnapshot {
    pub source_mod: Option<String>,
    pub scene_id: Option<String>,
    pub transition_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ActiveSceneTransitionState {
    plan: SceneTransitionPlan,
    consumed_transition_ids: Vec<String>,
    elapsed_seconds: f32,
}

#[derive(Debug, Default)]
struct SceneTransitionState {
    active: Option<ActiveSceneTransitionState>,
}

#[derive(Debug, Default)]
pub struct SceneTransitionService {
    state: Mutex<SceneTransitionState>,
}

impl SceneTransitionService {
    pub fn activate(&self, plan: Option<SceneTransitionPlan>) {
        let mut state = self
            .state
            .lock()
            .expect("scene transition state mutex should not be poisoned");
        state.active = plan.map(|plan| ActiveSceneTransitionState {
            plan,
            consumed_transition_ids: Vec::new(),
            elapsed_seconds: 0.0,
        });
    }

    pub fn clear(&self) {
        self.activate(None);
    }

    pub fn snapshot(&self) -> SceneTransitionSnapshot {
        let state = self
            .state
            .lock()
            .expect("scene transition state mutex should not be poisoned");
        let Some(active) = state.active.as_ref() else {
            return SceneTransitionSnapshot::default();
        };

        SceneTransitionSnapshot {
            source_mod: Some(active.plan.source_mod.clone()),
            scene_id: Some(active.plan.scene_id.clone()),
            transition_ids: active
                .plan
                .transitions
                .iter()
                .map(|transition| transition.id.clone())
                .collect(),
        }
    }

    pub fn tick(&self, delta_seconds: f32) -> Vec<SceneCommand> {
        let mut state = self
            .state
            .lock()
            .expect("scene transition state mutex should not be poisoned");
        let Some(active) = state.active.as_mut() else {
            return Vec::new();
        };

        active.elapsed_seconds += delta_seconds.max(0.0);
        let elapsed_seconds = active.elapsed_seconds;
        collect_triggered_commands(active, |transition| match &transition.trigger {
            SceneTransitionTrigger::AfterSeconds { seconds } => elapsed_seconds >= *seconds,
            SceneTransitionTrigger::ScriptEvent { .. } => false,
        })
    }

    pub fn observe_script_event(&self, topic: &str, payload: &[String]) -> Vec<SceneCommand> {
        let mut state = self
            .state
            .lock()
            .expect("scene transition state mutex should not be poisoned");
        let Some(active) = state.active.as_mut() else {
            return Vec::new();
        };

        collect_triggered_commands(active, |transition| match &transition.trigger {
            SceneTransitionTrigger::AfterSeconds { .. } => false,
            SceneTransitionTrigger::ScriptEvent {
                topic: expected_topic,
                payload: expected_payload,
            } => {
                expected_topic == topic
                    && (expected_payload.is_empty() || expected_payload.as_slice() == payload)
            }
        })
    }
}

pub fn build_scene_transition_plan(
    source_mod: &str,
    document: &SceneDocument,
) -> SceneDocumentResult<Option<SceneTransitionPlan>> {
    if document.transitions.is_empty() {
        return Ok(None);
    }

    let transitions = document
        .transitions
        .iter()
        .enumerate()
        .map(|(index, transition)| {
            let id = if transition.id.trim().is_empty() {
                format!("{}-transition-{}", document.scene.id, index + 1)
            } else {
                transition.id.clone()
            };

            if transition.to.trim().is_empty() {
                return Err(SceneDocumentError::Hydration {
                    scene_id: document.scene.id.clone(),
                    entity_id: "<scene>".to_owned(),
                    component_kind: "SceneTransition".to_owned(),
                    message: "scene transition target must not be empty".to_owned(),
                });
            }

            let trigger = match &transition.when {
                crate::SceneTransitionConditionDocument::AfterSeconds { seconds } => {
                    if *seconds <= 0.0 {
                        return Err(SceneDocumentError::Hydration {
                            scene_id: document.scene.id.clone(),
                            entity_id: "<scene>".to_owned(),
                            component_kind: "SceneTransition".to_owned(),
                            message: format!(
                                "scene transition `{id}` must use a positive `after_seconds` value"
                            ),
                        });
                    }

                    SceneTransitionTrigger::AfterSeconds { seconds: *seconds }
                }
                crate::SceneTransitionConditionDocument::ScriptEvent { topic, payload } => {
                    if topic.trim().is_empty() {
                        return Err(SceneDocumentError::Hydration {
                            scene_id: document.scene.id.clone(),
                            entity_id: "<scene>".to_owned(),
                            component_kind: "SceneTransition".to_owned(),
                            message: format!(
                                "scene transition `{id}` must declare a non-empty script event topic"
                            ),
                        });
                    }

                    SceneTransitionTrigger::ScriptEvent {
                        topic: topic.clone(),
                        payload: payload.clone(),
                    }
                }
            };

            Ok(SceneTransitionRule {
                id,
                target_scene: SceneKey::new(transition.to.clone()),
                trigger,
            })
        })
        .collect::<SceneDocumentResult<Vec<_>>>()?;

    Ok(Some(SceneTransitionPlan {
        source_mod: source_mod.to_owned(),
        scene_id: document.scene.id.clone(),
        transitions,
    }))
}

fn collect_triggered_commands(
    active: &mut ActiveSceneTransitionState,
    predicate: impl Fn(&SceneTransitionRule) -> bool,
) -> Vec<SceneCommand> {
    let mut commands = Vec::new();
    let transitions = active.plan.transitions.clone();

    for transition in transitions {
        if active
            .consumed_transition_ids
            .iter()
            .any(|transition_id| transition_id == &transition.id)
        {
            continue;
        }

        if predicate(&transition) {
            active.consumed_transition_ids.push(transition.id.clone());
            commands.push(SceneCommand::SelectScene {
                scene: transition.target_scene.clone(),
            });
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneDocument, SceneMetadataDocument, SceneTransitionConditionDocument,
        SceneTransitionDocument,
    };

    use super::{SceneTransitionService, SceneTransitionTrigger, build_scene_transition_plan};

    #[test]
    fn builds_scene_transition_plan_from_document() {
        let document = SceneDocument {
            version: 1,
            scene: SceneMetadataDocument {
                id: "intro".to_owned(),
                label: "Intro".to_owned(),
                description: None,
            },
            transitions: vec![
                SceneTransitionDocument {
                    id: "auto-next".to_owned(),
                    to: "main".to_owned(),
                    when: SceneTransitionConditionDocument::AfterSeconds { seconds: 2.0 },
                },
                SceneTransitionDocument {
                    id: "cutscene-end".to_owned(),
                    to: "main".to_owned(),
                    when: SceneTransitionConditionDocument::ScriptEvent {
                        topic: "cutscene.finished".to_owned(),
                        payload: vec!["intro".to_owned()],
                    },
                },
            ],
            entities: Vec::new(),
        };

        let plan = build_scene_transition_plan("demo-mod", &document)
            .expect("transition plan should build")
            .expect("transition plan should exist");

        assert_eq!(plan.scene_id, "intro");
        assert_eq!(plan.transitions.len(), 2);
        assert!(matches!(
            plan.transitions[0].trigger,
            SceneTransitionTrigger::AfterSeconds { seconds } if (seconds - 2.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn transition_service_triggers_after_seconds_once() {
        let service = SceneTransitionService::default();
        let document = SceneDocument {
            version: 1,
            scene: SceneMetadataDocument {
                id: "intro".to_owned(),
                label: String::new(),
                description: None,
            },
            transitions: vec![SceneTransitionDocument {
                id: "auto-next".to_owned(),
                to: "main".to_owned(),
                when: SceneTransitionConditionDocument::AfterSeconds { seconds: 1.0 },
            }],
            entities: Vec::new(),
        };
        let plan = build_scene_transition_plan("demo-mod", &document)
            .expect("transition plan should build");
        service.activate(plan);

        assert!(service.tick(0.5).is_empty());
        let commands = service.tick(0.5);
        assert_eq!(commands.len(), 1);
        assert!(service.tick(1.0).is_empty());
    }

    #[test]
    fn transition_service_triggers_on_script_event_once() {
        let service = SceneTransitionService::default();
        let document = SceneDocument {
            version: 1,
            scene: SceneMetadataDocument {
                id: "cutscene".to_owned(),
                label: String::new(),
                description: None,
            },
            transitions: vec![SceneTransitionDocument {
                id: "cutscene-end".to_owned(),
                to: "main".to_owned(),
                when: SceneTransitionConditionDocument::ScriptEvent {
                    topic: "cutscene.finished".to_owned(),
                    payload: vec!["intro".to_owned()],
                },
            }],
            entities: Vec::new(),
        };
        let plan = build_scene_transition_plan("demo-mod", &document)
            .expect("transition plan should build");
        service.activate(plan);

        assert!(
            service
                .observe_script_event("cutscene.finished", &[String::from("other")])
                .is_empty()
        );
        assert_eq!(
            service
                .observe_script_event("cutscene.finished", &[String::from("intro")])
                .len(),
            1
        );
        assert!(
            service
                .observe_script_event("cutscene.finished", &[String::from("intro")])
                .is_empty()
        );
    }
}
