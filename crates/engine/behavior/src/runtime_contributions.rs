use amigo_session::{
    domain_contributions::{
        RuntimeContributionDescriptor, RuntimeContributionKind, RuntimeDomainContribution,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
        SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.behavior";
const SCENE_HANDLER_ID: &str = "behavior";
const SCENE_CONTRIBUTION_ID: &str = "behavior.scene";
const SYSTEM_ID: &str = "behavior";
const SYSTEM_PHASE: &str = "update";

pub fn register_behavior_runtime_contributions(
    session: &mut RuntimeSession,
) -> (Vec<SceneCommandHandlerContribution>, Vec<SystemContribution>) {
    let scene_contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: SCENE_HANDLER_ID.to_string(),
        },
    }];
    let system_contributions = vec![SystemContribution {
        descriptor: system_descriptor(),
    }];

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
                    description: "Behavior system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }

    (scene_contributions, system_contributions)
}

fn scene_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "Behavior scene command handler".to_string(),
        capabilities: vec!["behavior".to_string()],
        tags: vec!["engine".to_string(), "behavior".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor() -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: SYSTEM_ID.to_string(),
        phase: SYSTEM_PHASE.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: "behavior.domain".to_string(),
        capabilities: vec!["behavior".to_string()],
        tags: vec!["engine".to_string(), "behavior".to_string()],
        migration_seam: false,
    }
}
