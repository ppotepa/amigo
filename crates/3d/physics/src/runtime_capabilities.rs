use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
        SceneCommandHandlerContribution, SceneCommandHandlerDescriptor, SystemContribution,
        SystemDescriptor,
    },
};

const DOMAIN_ID: &str = "amigo.3d.physics";
const BODY_SCENE_HANDLER_ID: &str = "body-3d";
const COLLIDER_SCENE_HANDLER_ID: &str = "collider-3d";
const SYSTEM_ID: &str = "physics_3d_step";
const SYSTEM_PHASE: &str = "update";

pub fn register_physics3d_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (
    Vec<SceneCommandHandlerContribution>,
    Vec<SystemContribution>,
) {
    let scene_contributions = vec![
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(BODY_SCENE_HANDLER_ID),
                handler_id: BODY_SCENE_HANDLER_ID.to_string(),
            },
        },
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(COLLIDER_SCENE_HANDLER_ID),
                handler_id: COLLIDER_SCENE_HANDLER_ID.to_string(),
            },
        },
    ];
    let system_contributions = vec![SystemContribution {
        descriptor: system_descriptor(),
    }];

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
                    description: "3D physics system phase handler".to_owned(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: false,
                },
            });
    }

    (scene_contributions, system_contributions)
}

fn scene_descriptor(handler_id: &str) -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: format!("{handler_id}.scene"),
        label: handler_id.to_owned(),
        description: "3D physics scene command handler".to_owned(),
        capabilities: vec!["physics_3d".to_owned()],
        tags: vec!["3d".to_owned(), "physics".to_owned()],
        migration_seam: false,
    }
}

fn system_descriptor() -> SystemDescriptor {
    SystemDescriptor {
        domain_id: RuntimeDomainId::new(DOMAIN_ID),
        system_id: SYSTEM_ID.to_owned(),
        phase: SYSTEM_PHASE.to_owned(),
        ordering: 0,
        main_thread_required: true,
        diagnostics_label: format!("{SYSTEM_ID}.domain"),
        capabilities: vec!["physics_3d".to_owned()],
        tags: vec!["3d".to_owned(), "physics".to_owned()],
        migration_seam: false,
    }
}
