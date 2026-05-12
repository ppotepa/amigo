use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.input-actions";
const SCENE_HANDLER_ID: &str = "input-actions";
const SCENE_CONTRIBUTION_ID: &str = "input-actions.scene";

pub fn register_input_actions_runtime_capabilities(
    session: &mut RuntimeSession,
) -> Vec<SceneCommandHandlerContribution> {
    let contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: SCENE_HANDLER_ID.to_string(),
        },
    }];

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
}

fn scene_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "Input actions scene command handler".to_string(),
        capabilities: vec!["input_actions".to_string()],
        tags: vec!["engine".to_string(), "input".to_string()],
        migration_seam: false,
    }
}
