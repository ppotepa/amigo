use amigo_session::{
    runtime_capabilities::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RuntimeCapabilityDescriptor,
        RuntimeCapabilityKind, RuntimeCapability, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor, SystemContribution,
        SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.engine.scene";
const LIFECYCLE_HANDLER_ID: &str = "lifecycle";
const ACTIVATION_HANDLER_ID: &str = "activation";
const CAMERA_2D_HANDLER_ID: &str = "camera-2d";
const LIFETIME_SYSTEM_ID: &str = "lifetime";
const SCENE_TRANSITION_SYSTEM_ID: &str = "scene_transition";
const CAMERA_FOLLOW_2D_SYSTEM_ID: &str = "camera_follow_2d";
const PARALLAX_2D_SYSTEM_ID: &str = "parallax_2d";
const UPDATE_PHASE: &str = "update";

pub fn register_scene_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<SystemContribution>,
    Vec<DevConsoleCommandContribution>,
) {
    let scene_contributions = vec![
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(LIFECYCLE_HANDLER_ID),
                handler_id: LIFECYCLE_HANDLER_ID.to_string(),
            },
        },
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(ACTIVATION_HANDLER_ID),
                handler_id: ACTIVATION_HANDLER_ID.to_string(),
            },
        },
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(CAMERA_2D_HANDLER_ID),
                handler_id: CAMERA_2D_HANDLER_ID.to_string(),
            },
        },
    ];
    let system_contributions = vec![
        SystemContribution {
            descriptor: system_descriptor(LIFETIME_SYSTEM_ID, "Scene lifetime system"),
        },
        SystemContribution {
            descriptor: system_descriptor(
                SCENE_TRANSITION_SYSTEM_ID,
                "Scene transition system",
            ),
        },
        SystemContribution {
            descriptor: system_descriptor(
                CAMERA_FOLLOW_2D_SYSTEM_ID,
                "Camera follow 2D system",
            ),
        },
        SystemContribution {
            descriptor: system_descriptor(PARALLAX_2D_SYSTEM_ID, "Parallax 2D system"),
        },
    ];
    let dev_console_contributions = vec![
        dev_console_contribution("scene.reload", "Reload the active scene."),
        dev_console_contribution("scene.select", "Select a scene by id."),
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
                    description: "Scene-owned system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
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
        system_contributions,
        dev_console_contributions,
    )
}

fn scene_descriptor(handler_id: &str) -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: format!("{handler_id}.scene"),
        label: handler_id.to_string(),
        description: "Scene-owned scene command handler".to_string(),
        capabilities: vec!["scene".to_string()],
        tags: vec!["engine".to_string(), "scene".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor(system_id: &str, diagnostics_label: &str) -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: system_id.to_string(),
        phase: UPDATE_PHASE.to_string(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: diagnostics_label.to_string(),
        capabilities: vec!["scene".to_string()],
        tags: vec!["engine".to_string(), "scene".to_string()],
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
                capabilities: vec!["scene".to_string()],
                tags: vec!["engine".to_string(), "scene".to_string()],
                migration_seam: false,
            },
        },
    }
}
