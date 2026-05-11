use amigo_session::{
    domain_contributions::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RuntimeContributionDescriptor,
        RuntimeContributionKind, RuntimeDomainContribution, RuntimeDomainId,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.assets";

pub fn register_assets_runtime_contributions(
    session: &mut RuntimeSession,
) -> Vec<DevConsoleCommandContribution> {
    let contributions = vec![
        dev_console_contribution("assets", "Show asset catalog summary."),
        dev_console_contribution("asset.reload", "Reload an asset by key."),
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
                capabilities: vec!["assets".to_string()],
                tags: vec!["engine".to_string(), "assets".to_string()],
                migration_seam: false,
            },
        },
    }
}
