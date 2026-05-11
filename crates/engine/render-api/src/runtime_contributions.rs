use amigo_session::{
    domain_contributions::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RuntimeContributionDescriptor,
        RuntimeContributionKind, RuntimeDomainContribution, RuntimeDomainId,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.render-api";

pub fn register_render_runtime_contributions(
    session: &mut RuntimeSession,
) -> Vec<DevConsoleCommandContribution> {
    let contributions = vec![
        dev_console_contribution("render.stats", "Show current render frame stats."),
        dev_console_contribution("render.plan", "Show resolved frame composition plan."),
        dev_console_contribution("render.graph", "Show resolved frame graph nodes."),
    ];

    for contribution in &contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
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
                capabilities: vec!["render".to_string()],
                tags: vec!["engine".to_string(), "render".to_string()],
                migration_seam: false,
            },
        },
    }
}
