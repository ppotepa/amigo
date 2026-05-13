use amigo_session::{
    runtime_capabilities::{
        RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    },
    RuntimeSession,
};

pub const CAMERA_DOMAIN_ID: &str = "amigo.camera";

const CAMERA_2D_HANDLER_ID: &str = "camera-2d";
const CAMERA_FOLLOW_2D_SYSTEM_ID: &str = "camera_follow_2d";
const PARALLAX_2D_SYSTEM_ID: &str = "parallax_2d";
const UPDATE_PHASE: &str = "update";

pub fn register_camera_runtime_capabilities(session: &mut RuntimeSession) {
    for descriptor in [
        scene_handler_descriptor(CAMERA_2D_HANDLER_ID),
        system_descriptor(CAMERA_FOLLOW_2D_SYSTEM_ID, "Camera follow 2D system"),
        system_descriptor(PARALLAX_2D_SYSTEM_ID, "Parallax 2D system"),
    ] {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability { descriptor });
    }
}

fn scene_handler_descriptor(handler_id: &str) -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(CAMERA_DOMAIN_ID),
        kind: RuntimeCapabilityKind::SceneCommandHandler,
        id: format!("{handler_id}.scene"),
        label: handler_id.to_string(),
        description: "Camera-owned scene command handler".to_string(),
        capabilities: vec!["scene".to_string(), "camera".to_string()],
        tags: vec!["engine".to_string(), "camera".to_string()],
        migration_seam: false,
    }
}

fn system_descriptor(system_id: &str, diagnostics_label: &str) -> RuntimeCapabilityDescriptor {
    RuntimeCapabilityDescriptor {
        domain_id: RuntimeDomainId::new(CAMERA_DOMAIN_ID),
        kind: RuntimeCapabilityKind::SystemPhaseHandler,
        id: format!("{system_id}.{UPDATE_PHASE}"),
        label: format!("System {system_id}"),
        description: diagnostics_label.to_string(),
        capabilities: vec!["scene".to_string(), "camera".to_string()],
        tags: vec!["engine".to_string(), "camera".to_string()],
        migration_seam: false,
    }
}

