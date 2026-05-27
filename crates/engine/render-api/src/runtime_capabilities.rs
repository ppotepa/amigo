use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        DevConsoleCommandContribution, DevConsoleCommandDescriptor, RuntimeCapability,
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeDomainId,
    },
};

const DOMAIN_ID: &str = "amigo.engine.render-api";

pub fn register_render_runtime_capabilities(
    session: &mut RuntimeSession,
) -> Vec<DevConsoleCommandContribution> {
    let contributions = vec![
        dev_console_contribution("render.stats", "Show current render frame stats."),
        dev_console_contribution("render.plan", "Show resolved frame composition plan."),
        dev_console_contribution("render.graph", "Show resolved frame graph nodes."),
        dev_console_contribution("camera.capture", "Show resolved 2D camera capture input."),
        dev_console_contribution(
            "camera.focus.plan",
            "Show resolved 2D camera focus/depth plan.",
        ),
    ];

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
}

fn dev_console_contribution(id: &str, description: &str) -> DevConsoleCommandContribution {
    DevConsoleCommandContribution {
        descriptor: DevConsoleCommandDescriptor {
            descriptor: RuntimeCapabilityDescriptor {
                domain_id: RuntimeDomainId::new(DOMAIN_ID),
                kind: RuntimeCapabilityKind::DevConsoleCommand,
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
