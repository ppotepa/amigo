use amigo_session::{
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RuntimeCapability,
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.tilemap";
const SCENE_HANDLER_ID: &str = "tilemap-2d";
const SCENE_CONTRIBUTION_ID: &str = "tilemap-2d.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_tilemap_2d";

pub fn register_tilemap2d_runtime_capabilities(
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
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "2D tilemap scene command handler".to_string(),
        capabilities: vec!["tilemap_2d".to_string()],
        tags: vec!["2d".to_string(), "tilemap".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "TileMap 2D Extractor".to_string(),
        description: "2D tilemap render extractor".to_string(),
        capabilities: vec!["tilemap_2d".to_string()],
        tags: vec!["2d".to_string(), "tilemap".to_string()],
        migration_seam: false,
    }
}
