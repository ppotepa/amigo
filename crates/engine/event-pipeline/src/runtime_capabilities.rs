use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
};

const DOMAIN_ID: &str = "amigo.engine.event-pipeline";
const SCENE_HANDLER_ID: &str = "event-pipeline";
const SCENE_CONTRIBUTION_ID: &str = "event-pipeline.scene";

pub fn register_event_pipeline_runtime_capabilities(
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
        description: "Event pipeline scene command handler".to_string(),
        capabilities: vec!["event_pipeline".to_string()],
        tags: vec!["engine".to_string(), "events".to_string()],
        migration_seam: false,
    }
}
