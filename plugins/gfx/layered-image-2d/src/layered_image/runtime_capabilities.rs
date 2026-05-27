use amigo_session::{
    runtime_capabilities::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RenderExtractorContribution,
        RenderExtractorDescriptor, RuntimeCapability, RuntimeCapabilityDescriptor,
        RuntimeCapabilityKind, RuntimeDomainId, SceneCommandHandlerContribution,
        SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.layered-image";
const SCENE_HANDLER_ID: &str = "layered-image-2d";
const SCENE_CONTRIBUTION_ID: &str = "layered-image-2d.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_layered_image_2d";

pub fn register_layered_image_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<RenderExtractorContribution>,
    Vec<DevConsoleCommandContribution>,
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
    let dev_console_contributions = vec![dev_console_contribution(
        "layered.opacity",
        "Set runtime opacity for one layered image layer.",
    )];

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
    for contribution in &dev_console_contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    (
        scene_contributions,
        render_contributions,
        dev_console_contributions,
    )
}

fn scene_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "2D layered image scene command handler".to_string(),
        capabilities: vec!["rendering_2d".to_string()],
        tags: vec!["2d".to_string(), "layered-image".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "Layered Image 2D Extractor".to_string(),
        description: "2D layered image render extractor".to_string(),
        capabilities: vec!["rendering_2d".to_string()],
        tags: vec!["2d".to_string(), "layered-image".to_string()],
        migration_seam: false,
    }
}

fn dev_console_contribution(id: &str, description: &str) -> DevConsoleCommandContribution {
    DevConsoleCommandContribution {
        descriptor: DevConsoleCommandDescriptor {
            descriptor: RuntimeCapabilityDescriptor {
                domain_id: RuntimeDomainId::new(DOMAIN_ID),
                kind: RuntimeCapabilityKind::DevConsoleCommand,
                id: id.to_string(),
                label: id.to_string(),
                description: description.to_string(),
                capabilities: vec!["rendering_2d".to_string()],
                tags: vec!["2d".to_string(), "layered-image".to_string()],
                migration_seam: false,
            },
        },
    }
}
