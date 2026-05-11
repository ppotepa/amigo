use amigo_session::{
    domain_contributions::{
        RuntimeContributionDescriptor, RuntimeContributionKind, RuntimeDomainContribution,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
        SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.ui.core";
const UI_SCENE_HANDLER_ID: &str = "ui";
const UI_MODEL_BINDINGS_SCENE_HANDLER_ID: &str = "ui-model-bindings";
const UI_INPUT_SYSTEM_ID: &str = "ui_input";
const UI_INPUT_PHASE: &str = "pre_update";
const UI_BINDINGS_SYSTEM_ID: &str = "ui_bindings";
const UI_BINDINGS_PHASE: &str = "update";

pub fn register_ui_runtime_contributions(
    session: &mut RuntimeSession,
) -> (Vec<SceneCommandHandlerContribution>, Vec<SystemContribution>) {
    let scene_contributions = vec![
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(UI_SCENE_HANDLER_ID, "UI scene command handler"),
                handler_id: UI_SCENE_HANDLER_ID.to_string(),
            },
        },
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(
                    UI_MODEL_BINDINGS_SCENE_HANDLER_ID,
                    "UI model bindings scene command handler",
                ),
                handler_id: UI_MODEL_BINDINGS_SCENE_HANDLER_ID.to_string(),
            },
        },
    ];
    let system_contributions = vec![
        SystemContribution {
            descriptor: system_descriptor(
                UI_INPUT_SYSTEM_ID,
                UI_INPUT_PHASE,
                "ui_input.domain",
                "ui_input",
            ),
        },
        SystemContribution {
            descriptor: system_descriptor(
                UI_BINDINGS_SYSTEM_ID,
                UI_BINDINGS_PHASE,
                "ui_bindings.domain",
                "ui_bindings",
            ),
        },
    ];

    for contribution in &scene_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }
    for contribution in &system_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: RuntimeContributionDescriptor {
                    domain_id: RuntimeDomainId::new(DOMAIN_ID),
                    kind: RuntimeContributionKind::SystemPhaseHandler,
                    id: format!(
                        "{}.{}",
                        contribution.descriptor.system_id, contribution.descriptor.phase
                    ),
                    label: format!("System {}", contribution.descriptor.system_id),
                    description: "UI system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }

    (scene_contributions, system_contributions)
}

fn scene_descriptor(handler_id: &str, description: &str) -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: format!("{handler_id}.scene"),
        label: handler_id.to_string(),
        description: description.to_string(),
        capabilities: vec!["ui".to_string()],
        tags: vec!["ui".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor(
    system_id: &str,
    phase: &str,
    diagnostics_label: &str,
    capability: &str,
) -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: system_id.to_string(),
        phase: phase.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: diagnostics_label.to_string(),
        capabilities: vec![capability.to_string()],
        tags: vec!["ui".to_string()],
        migration_seam: false,
    }
}
