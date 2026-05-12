use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.audio.mixer";
const SYSTEM_ID: &str = "audio_runtime";
const SYSTEM_PHASE: &str = "post_update";

pub fn register_audio_mixer_runtime_capabilities(
    session: &mut RuntimeSession,
) -> Vec<SystemContribution> {
    let contributions = vec![SystemContribution {
        descriptor: system_descriptor(),
    }];

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new(DOMAIN_ID),
                    kind: RuntimeCapabilityKind::SystemPhaseHandler,
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

fn diagnostics_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::DiagnosticsProvider,
        id: "audio.mixer.diagnostics".to_owned(),
        label: "Audio mixer diagnostics".to_owned(),
        description: "Audio mixer runtime diagnostics owned by the audio mixer domain".to_owned(),
        capabilities: vec!["diagnostics".to_owned(), "audio".to_owned()],
        tags: vec!["audio".to_owned(), "mixer".to_owned()],
        migration_seam: false,
    }
}

fn metadata_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::MetadataProvider,
        id: "audio.mixer.metadata".to_owned(),
        label: "Audio mixer metadata".to_owned(),
        description: "Audio mixer runtime metadata owned by the audio mixer domain".to_owned(),
        capabilities: vec!["metadata".to_owned(), "audio".to_owned()],
        tags: vec!["audio".to_owned(), "mixer".to_owned()],
        migration_seam: false,
    }
}
