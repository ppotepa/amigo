use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        RenderExtractorContribution, RenderExtractorDescriptor, RuntimeCapability,
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
    },
};

const DOMAIN_ID: &str = "amigo.2d.depth-map";
const SCENE_HANDLER_ID: &str = "depth-map-2d";
const SCENE_CONTRIBUTION_ID: &str = "depth-map-2d.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_depth_map_2d";

pub fn register_depth_map_runtime_capabilities(
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
        description: "2D depth map scene command handler".to_string(),
        capabilities: vec!["rendering_2d".to_string(), "camera_2d".to_string()],
        tags: vec![
            "2d".to_string(),
            "depth-map".to_string(),
            "camera".to_string(),
        ],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "Depth Map 2D Extractor".to_string(),
        description: "2D depth map render extractor for camera depth of field".to_string(),
        capabilities: vec!["rendering_2d".to_string(), "camera_2d".to_string()],
        tags: vec![
            "2d".to_string(),
            "depth-map".to_string(),
            "camera".to_string(),
        ],
        migration_seam: false,
    }
}
