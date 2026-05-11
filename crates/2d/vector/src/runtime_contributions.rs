use amigo_session::{
    domain_contributions::{
        RenderExtractorContribution, RenderExtractorDescriptor, RuntimeContributionDescriptor,
        RuntimeContributionKind, RuntimeDomainContribution, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const VECTOR_DOMAIN_ID: &str = "amigo.2d.vector";
const VECTOR_SCENE_HANDLER_ID: &str = "vector-2d";
const VECTOR_SCENE_CONTRIBUTION_ID: &str = "vector-2d.scene";
const VECTOR_RENDER_EXTRACTOR_ID: &str = "resolved_vector_2d";

pub fn register_vector2d_runtime_contributions(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<RenderExtractorContribution>,
) {
    let scene_contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: VECTOR_SCENE_HANDLER_ID.to_string(),
        },
    }];
    let render_contributions = vec![RenderExtractorContribution {
        descriptor: RenderExtractorDescriptor {
            descriptor: render_descriptor(),
        },
    }];

    for contribution in &scene_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    for contribution in &render_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    (scene_contributions, render_contributions)
}

fn scene_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(VECTOR_DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: VECTOR_SCENE_CONTRIBUTION_ID.to_string(),
        label: "vector-2d".to_string(),
        description: "2D vector scene command handler".to_string(),
        capabilities: vec!["vector_2d".to_string()],
        tags: vec!["2d".to_string(), "vector".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(VECTOR_DOMAIN_ID),
        kind: RuntimeContributionKind::RenderExtractor,
        id: VECTOR_RENDER_EXTRACTOR_ID.to_string(),
        label: "Vector 2D Extractor".to_string(),
        description: "2D vector render extractor".to_string(),
        capabilities: vec!["vector_2d".to_string()],
        tags: vec!["2d".to_string(), "vector".to_string()],
        migration_seam: false,
    }
}
