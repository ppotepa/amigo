use amigo_session::{
    domain_contributions::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RenderExtractorContribution,
        RenderExtractorDescriptor, RuntimeContributionDescriptor, RuntimeContributionKind,
        RuntimeDomainContribution, RuntimeDomainId, SceneCommandHandlerContribution,
        SceneCommandHandlerDescriptor, SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.particles";
const SCENE_HANDLER_ID: &str = "particles-2d";
const SCENE_CONTRIBUTION_ID: &str = "particles-2d.scene";
const RENDER_EXTRACTOR_ID: &str = "resolved_particle_2d";
const SYSTEM_ID: &str = "particles_2d";
const SYSTEM_PHASE: &str = "update";

pub fn register_particles2d_runtime_contributions(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<RenderExtractorContribution>,
    Vec<SystemContribution>,
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
    let system_contributions = vec![SystemContribution {
        descriptor: system_descriptor(),
    }];
    let dev_console_contributions = vec![
        dev_console_contribution("particles.list", "List particle emitters."),
        dev_console_contribution("particles.pause", "Disable all particle emitters."),
        dev_console_contribution(
            "particles.emitters",
            "Show emitter live counts and effective budget.",
        ),
        dev_console_contribution(
            "particles.budget",
            "Set a temporary global particle budget multiplier.",
        ),
    ];

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
    for contribution in &system_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: RuntimeContributionDescriptor {
                    domain_id: RuntimeDomainId::new(DOMAIN_ID),
                    kind: RuntimeContributionKind::SystemPhaseHandler,
                    id: format!("{}.{}", contribution.descriptor.system_id, contribution.descriptor.phase),
                    label: format!("System {}", contribution.descriptor.system_id),
                    description: "2D particles system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }
    for contribution in &dev_console_contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    (
        scene_contributions,
        render_contributions,
        system_contributions,
        dev_console_contributions,
    )
}

fn scene_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::SceneCommandHandler,
        id: SCENE_CONTRIBUTION_ID.to_string(),
        label: SCENE_HANDLER_ID.to_string(),
        description: "2D particles scene command handler".to_string(),
        capabilities: vec!["particles_2d".to_string()],
        tags: vec!["2d".to_string(), "particles".to_string()],
        migration_seam: false,
    }
}

fn render_descriptor() -> RuntimeContributionDescriptor {
    RuntimeContributionDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeContributionKind::RenderExtractor,
        id: RENDER_EXTRACTOR_ID.to_string(),
        label: "Particle 2D Extractor".to_string(),
        description: "2D particles render extractor".to_string(),
        capabilities: vec!["particles_2d".to_string()],
        tags: vec!["2d".to_string(), "particles".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor() -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: SYSTEM_ID.to_string(),
        phase: SYSTEM_PHASE.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: format!("{SYSTEM_ID}.domain"),
        capabilities: vec!["particles_2d".to_string()],
        tags: vec!["2d".to_string(), "particles".to_string()],
        migration_seam: false,
    }
}

fn dev_console_contribution(
    id: &str,
    description: &str,
) -> DevConsoleCommandContribution {
    DevConsoleCommandContribution {
        descriptor: DevConsoleCommandDescriptor {
            descriptor: RuntimeContributionDescriptor {
                domain_id: RuntimeDomainId::new(DOMAIN_ID),
                kind: RuntimeContributionKind::DevConsoleCommand,
                id: id.to_string(),
                label: id.to_string(),
                description: description.to_string(),
                capabilities: vec!["particles_2d".to_string()],
                tags: vec!["2d".to_string(), "particles".to_string()],
                migration_seam: false,
            },
        },
    }
}
