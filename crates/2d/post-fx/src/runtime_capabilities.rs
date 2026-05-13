use amigo_session::{
    runtime_capabilities::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RenderExtractorContribution,
        RenderExtractorDescriptor, RuntimeCapabilityDescriptor, RuntimeCapabilityKind,
        RuntimeCapability, RuntimeDomainId, SceneCommandHandlerContribution,
        SceneCommandHandlerDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.post-fx";
const SCENE_HANDLER_ID: &str = "post-fx";
const SCENE_CONTRIBUTION_ID: &str = "post-fx.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_postfx_2d";

pub fn register_post_fx_runtime_capabilities(
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
    let dev_console_contributions = vec![
        dev_console_contribution("postfx.cert", "Show LensDroplets2D certification reports."),
        dev_console_contribution("postfx.stats", "Show active 2D post-fx stack stats."),
    ];

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
    session
        .runtime_capabilities_mut()
        .register(RuntimeCapability {
            descriptor: diagnostics_descriptor(),
        });
    session
        .runtime_capabilities_mut()
        .register(RuntimeCapability {
            descriptor: metadata_descriptor(),
        });

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
        description: "2D post-fx scene command handler".to_string(),
        capabilities: vec!["post_fx_2d".to_string(), "film_noise_2d".to_string()],
        tags: vec!["2d".to_string(), "post-fx".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "PostFx 2D Extractor".to_string(),
        description: "2D post-fx render extractor".to_string(),
        capabilities: vec!["post_fx_2d".to_string(), "film_noise_2d".to_string()],
        tags: vec!["2d".to_string(), "post-fx".to_string()],
        migration_seam: false,
    }
}

fn dev_console_contribution(
    id: &str,
    description: &str,
) -> DevConsoleCommandContribution {
    DevConsoleCommandContribution {
        descriptor: DevConsoleCommandDescriptor {
            descriptor: RuntimeCapabilityDescriptor {
                domain_id: RuntimeDomainId::new(DOMAIN_ID),
                kind: RuntimeCapabilityKind::DevConsoleCommand,
                id: id.to_string(),
                label: id.to_string(),
                description: description.to_string(),
                capabilities: vec!["post_fx_2d".to_string()],
                tags: vec!["2d".to_string(), "post-fx".to_string()],
                migration_seam: false,
            },
        },
    }
}

fn diagnostics_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::DiagnosticsProvider,
        id: "2d.post-fx.diagnostics".to_owned(),
        label: "2D post-fx diagnostics".to_owned(),
        description: "Post-fx runtime diagnostics owned by the 2D post-fx domain".to_owned(),
        capabilities: vec!["diagnostics".to_owned(), "post-fx".to_owned()],
        tags: vec!["2d".to_owned(), "post-fx".to_owned()],
        migration_seam: false,
    }
}

fn metadata_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::MetadataProvider,
        id: "2d.post-fx.metadata".to_owned(),
        label: "2D post-fx metadata".to_owned(),
        description: "Post-fx runtime metadata owned by the 2D post-fx domain".to_owned(),
        capabilities: vec!["metadata".to_owned(), "post-fx".to_owned()],
        tags: vec!["2d".to_owned(), "post-fx".to_owned()],
        migration_seam: false,
    }
}

