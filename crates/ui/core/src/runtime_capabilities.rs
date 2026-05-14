use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor, SystemContribution,
        SystemDescriptor,
    },
};

const DOMAIN_ID: &str = "amigo.ui.core";
const UI_SCENE_HANDLER_ID: &str = "ui";
const UI_MODEL_BINDINGS_SCENE_HANDLER_ID: &str = "ui-model-bindings";
const UI_INPUT_SYSTEM_ID: &str = "ui_input";
const UI_INPUT_PHASE: &str = "pre_update";
const UI_BINDINGS_SYSTEM_ID: &str = "ui_bindings";
const UI_BINDINGS_PHASE: &str = "update";

pub fn register_ui_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<SystemContribution>,
) {
    let scene_contributions = vec![
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(UI_SCENE_HANDLER_ID, "UI scene command handler"),
                handler_id: UI_SCENE_HANDLER_ID.to_string(),
            },
        },
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(
                    UI_MODEL_BINDINGS_SCENE_HANDLER_ID,
                    "UI model bindings scene command handler",
                ),
                handler_id: UI_MODEL_BINDINGS_SCENE_HANDLER_ID.to_string(),
            },
        },
    ];
    let system_contributions = vec![
        SystemContribution {
            descriptor: system_descriptor(
                UI_INPUT_SYSTEM_ID,
                UI_INPUT_PHASE,
                "ui_input.domain",
                "ui_input",
            ),
        },
        SystemContribution {
            descriptor: system_descriptor(
                UI_BINDINGS_SYSTEM_ID,
                UI_BINDINGS_PHASE,
                "ui_bindings.domain",
                "ui_bindings",
            ),
        },
    ];

    for contribution in &scene_contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }
    for contribution in &system_contributions {
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
                    description: "UI system phase handler".to_string(),
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

    (scene_contributions, system_contributions)
}

fn scene_descriptor(handler_id: &str, description: &str) -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: format!("{handler_id}.scene"),
        label: handler_id.to_string(),
        description: description.to_string(),
        capabilities: vec!["ui".to_string()],
        tags: vec!["ui".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor(
    system_id: &str,
    phase: &str,
    diagnostics_label: &str,
    capability: &str,
) -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: system_id.to_string(),
        phase: phase.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: diagnostics_label.to_string(),
        capabilities: vec![capability.to_string()],
        tags: vec!["ui".to_string()],
        migration_seam: false,
    }
}

fn diagnostics_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::DiagnosticsProvider,
        id: "ui.core.diagnostics".to_owned(),
        label: "UI core diagnostics".to_owned(),
        description: "UI runtime diagnostics owned by the UI core domain".to_owned(),
        capabilities: vec!["diagnostics".to_owned(), "ui".to_owned()],
        tags: vec!["ui".to_owned()],
        migration_seam: false,
    }
}

fn metadata_descriptor() -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::MetadataProvider,
        id: "ui.core.metadata".to_owned(),
        label: "UI core metadata".to_owned(),
        description: "UI runtime metadata owned by the UI core domain".to_owned(),
        capabilities: vec!["metadata".to_owned(), "ui".to_owned()],
        tags: vec!["ui".to_owned()],
        migration_seam: false,
    }
}
