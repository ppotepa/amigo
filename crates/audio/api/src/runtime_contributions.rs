use amigo_session::{
    domain_contributions::{
        RuntimeContributionDescriptor, RuntimeContributionKind, RuntimeDomainContribution,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.audio.api";
const SCENE_HANDLER_ID: &str = "audio";
const SCENE_CONTRIBUTION_ID: &str = "audio.scene";

pub fn register_audio_runtime_contributions(
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
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
}

fn scene_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "Audio scene command handler".to_string(),
        capabilities: vec!["audio".to_string()],
        tags: vec!["audio".to_string(), "scene".to_string()],
        migration_seam: false,
    }
}
