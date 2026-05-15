use amigo_session::{
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RuntimeCapability,
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.beacon";
const SCENE_HANDLER_ID: &str = "beacon-2d";
const SCENE_CONTRIBUTION_ID: &str = "beacon-2d.scene";
const RENDER_EXTRACTOR_ID: &str = "beacon_2d.vfx";

pub fn register_beacon2d_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<RenderExtractorContribution>,
) {
    let scene_contributions = vec![SceneCommandHandlerContribution {
        descriptor: SceneCommandHandlerDescriptor {
            descriptor: scene_descriptor(),
            handler_id: SCENE_HANDLER_ID.to_owned(),
        },
    }];
    let render_contributions = vec![RenderExtractorContribution {
        descriptor: RenderExtractorDescriptor {
            descriptor: render_descriptor(),
        },
    }];
    for contribution in &scene_contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }
    for contribution in &render_contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }
    (scene_contributions, render_contributions)
}

fn scene_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_owned(),
        label: "beacon-2d".to_owned(),
        description: "2D beacon light scene command handler".to_owned(),
        capabilities: vec!["beacon_2d".to_owned()],
        tags: vec!["2d".to_owned(), "beacon".to_owned(), "lighting".to_owned()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_owned(),
        label: "Beacon 2D Extractor".to_owned(),
        description: "2D beacon VFX render extraction".to_owned(),
        capabilities: vec!["beacon_2d".to_owned()],
        tags: vec!["2d".to_owned(), "beacon".to_owned(), "lighting".to_owned()],
        migration_seam: false,
    }
}
