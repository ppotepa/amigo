use amigo_session::{
    domain_contributions::{
        RuntimeContributionDescriptor, RuntimeContributionKind, RuntimeDomainContribution,
        RuntimeDomainId, SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.audio.mixer";
const SYSTEM_ID: &str = "audio_runtime";
const SYSTEM_PHASE: &str = "post_update";

pub fn register_audio_mixer_runtime_contributions(
    session: &mut RuntimeSession,
) -> Vec<SystemContribution> {
    let contributions = vec![SystemContribution {
        descriptor: system_descriptor(),
    }];

    for contribution in &contributions {
        session
            .domain_contributions_mut()
            .register(RuntimeDomainContribution {
                descriptor: RuntimeContributionDescriptor {
                    domain_id: RuntimeDomainId::new(DOMAIN_ID),
                    kind: RuntimeContributionKind::SystemPhaseHandler,
                    id: format!(
                        "{}.{}",
                        contribution.descriptor.system_id, contribution.descriptor.phase
                    ),
                    label: format!("System {}", contribution.descriptor.system_id),
                    description: "Audio mixer system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }

    contributions
}

fn system_descriptor() -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: SYSTEM_ID.to_string(),
        phase: SYSTEM_PHASE.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: "audio_runtime.domain".to_string(),
        capabilities: vec!["audio_mix".to_string()],
        tags: vec!["audio".to_string(), "mixer".to_string()],
        migration_seam: false,
    }
}
