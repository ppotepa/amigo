use amigo_session::{
    domain_contributions::{
        RuntimeContributionDescriptor, RuntimeContributionKind, RuntimeDomainContribution,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
        ScriptCommandHandlerContribution, ScriptCommandHandlerDescriptor, SystemContribution,
        SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.scripting.rhai";
const SCENE_HANDLER_ID: &str = "script-component";
const SCENE_CONTRIBUTION_ID: &str = "script-component.scene";
const SCRIPT_COMMAND_HANDLER_IDS: [&str; 5] = ["scene", "render", "asset", "audio", "ui"];
const SCRIPT_COMPONENTS_SYSTEM_ID: &str = "script_components";
const SCRIPT_UPDATE_SYSTEM_ID: &str = "script_update";
const UPDATE_PHASE: &str = "update";

pub fn register_rhai_runtime_contributions(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<SystemContribution>,
    Vec<ScriptCommandHandlerContribution>,
) {
    let scene_contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: SCENE_HANDLER_ID.to_string(),
        },
    }];
    let system_contributions = vec![
        SystemContribution {
            descriptor: system_descriptor(
                SCRIPT_COMPONENTS_SYSTEM_ID,
                "script_components.domain",
                "script_components",
            ),
        },
        SystemContribution {
            descriptor: system_descriptor(
                SCRIPT_UPDATE_SYSTEM_ID,
                "script_update.domain",
                "script_update",
            ),
        },
    ];
    let script_command_contributions = SCRIPT_COMMAND_HANDLER_IDS
        .into_iter()
        .map(|handler_id| ScriptCommandHandlerContribution {
            descriptor: ScriptCommandHandlerDescriptor {
                descriptor: script_command_descriptor(handler_id),
                handler_id: handler_id.to_string(),
            },
        })
        .collect::<Vec<_>>();

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
                    description: "Rhai scripting system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }
    for contribution in &script_command_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    (
        scene_contributions,
        system_contributions,
        script_command_contributions,
    )
}

fn scene_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "Rhai script component scene command handler".to_string(),
        capabilities: vec!["script_component".to_string()],
        tags: vec!["scripting".to_string(), "rhai".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor(
    system_id: &str,
    diagnostics_label: &str,
    capability: &str,
) -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: system_id.to_string(),
        phase: UPDATE_PHASE.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: diagnostics_label.to_string(),
        capabilities: vec![capability.to_string()],
        tags: vec!["scripting".to_string(), "rhai".to_string()],
        migration_seam: false,
    }
}

fn script_command_descriptor(handler_id: &str) -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::ScriptCommandHandler,
        id: format!("{handler_id}.script"),
        label: handler_id.to_string(),
        description: "Rhai-owned script command handler".to_string(),
        capabilities: vec![format!("{handler_id}_script_command")],
        tags: vec!["scripting".to_string(), "rhai".to_string()],
        migration_seam: false,
    }
}
