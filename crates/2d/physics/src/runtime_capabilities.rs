use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, SceneCommandHandlerContribution, SceneCommandHandlerDescriptor,
        SystemContribution, SystemDescriptor,
    },
    RuntimeSession,
};

const DOMAIN_ID: &str = "amigo.2d.physics";
const BODY_SCENE_HANDLER_ID: &str = "body-2d";
const COLLIDER_SCENE_HANDLER_ID: &str = "collider-2d";
const TRIGGER_SCENE_HANDLER_ID: &str = "trigger-2d";
const SYSTEM_ID: &str = "collision_events_2d";
const SYSTEM_PHASE: &str = "update";

pub fn register_physics2d_runtime_capabilities(
    session: &mut RuntimeSession,
) -> (Vec<SceneCommandHandlerContribution>, Vec<SystemContribution>) {
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
        SceneCommandHandlerContribution {
            descriptor: SceneCommandHandlerDescriptor {
                descriptor: scene_descriptor(TRIGGER_SCENE_HANDLER_ID),
                handler_id: TRIGGER_SCENE_HANDLER_ID.to_string(),
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
                    id: format!("{}.{}", contribution.descriptor.system_id, contribution.descriptor.phase),
                    label: format!("System {}", contribution.descriptor.system_id),
                    description: "2D physics system phase handler".to_string(),
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
        label: handler_id.to_string(),
        description: "2D physics scene command handler".to_string(),
        capabilities: vec!["physics_2d".to_string()],
        tags: vec!["2d".to_string(), "physics".to_string()],
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
        capabilities: vec!["physics_2d".to_string()],
        tags: vec!["2d".to_string(), "physics".to_string()],
        migration_seam: false,
    }
}

