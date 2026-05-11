use crate::{
    domain_contributions::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RuntimeContributionDescriptor,
        RuntimeContributionKind, RuntimeDomainContribution, RuntimeDomainId,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.session";

pub fn register_session_runtime_contributions(
    session: &mut RuntimeSession,
) -> Vec<DevConsoleCommandContribution> {
    let contributions = vec![
        dev_console_contribution("scheduler.stats", "Show current scheduler stats."),
        dev_console_contribution("scheduler.mode", "Show current scheduler mode."),
        dev_console_contribution(
            "scheduler.overrides",
            "Show resolved scheduling override diagnostics.",
        ),
        dev_console_contribution(
            "scheduler.set",
            "Set scheduler mode: single_thread|auto|hybrid|manual.",
        ),
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
                capabilities: vec!["scheduler".to_string()],
                tags: vec!["engine".to_string(), "session".to_string(), "scheduler".to_string()],
                migration_seam: false,
            },
        },
    }
}
