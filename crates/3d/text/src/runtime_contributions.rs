use amigo_session::{
    domain_contributions::{
        RenderExtractorContribution, RenderExtractorDescriptor, RuntimeContributionDescriptor,
        RuntimeContributionKind, RuntimeDomainContribution, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.3d.text";
const SCENE_HANDLER_ID: &str = "text-3d";
const SCENE_CONTRIBUTION_ID: &str = "text-3d.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_text_3d";

pub fn register_text3d_runtime_contributions(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<RenderExtractorContribution>,
) {
    let scene_contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: SCENE_HANDLER_ID.to_string(),
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
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "3D text scene command handler".to_string(),
        capabilities: vec!["text_3d".to_string()],
        tags: vec!["3d".to_string(), "text".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "Text 3D Extractor".to_string(),
        description: "3D text render extractor".to_string(),
        capabilities: vec!["text_3d".to_string()],
        tags: vec!["3d".to_string(), "text".to_string()],
        migration_seam: false,
    }
}
