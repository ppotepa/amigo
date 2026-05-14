use amigo_session::{
    RuntimeSession,
    runtime_capabilities::{
        APP_HOST_DOMAIN_ID, RuntimeCapability, RuntimeCapabilityDescriptor, RuntimeCapabilityKind,
        RuntimeDomainId,
    },
};

use crate::ConsoleCommandDescriptor;

pub fn register_console_command_capabilities<I>(session: &mut RuntimeSession, descriptors: I)
where
    I: IntoIterator<Item = ConsoleCommandDescriptor>,
{
    for descriptor in descriptors {
        if matches!(
            descriptor.category,
            "scene" | "assets" | "particles" | "layered-image" | "lighting" | "composition"
        ) || descriptor.name.starts_with("postfx.")
            || descriptor.name.starts_with("render.")
            || descriptor.name.starts_with("scheduler.")
        {
            continue;
        }

        let is_host_category = matches!(descriptor.category, "core" | "debug");
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new(APP_HOST_DOMAIN_ID),
                    kind: RuntimeCapabilityKind::DevConsoleCommand,
                    id: descriptor.name.to_string(),
                    label: descriptor.name.to_string(),
                    description: descriptor.help.to_string(),
                    capabilities: Vec::new(),
                    tags: vec![
                        "app".to_string(),
                        descriptor.category.to_string(),
                        if is_host_category {
                            "host".to_string()
                        } else {
                            "legacy".to_string()
                        },
                    ],
                    migration_seam: !is_host_category,
                },
            });
    }
}

pub fn register_devtools_capabilities(session: &mut RuntimeSession) {
    let console_registry = crate::ConsoleCommandRegistry::default();
    crate::commands::register_builtin_console_commands(&console_registry);
    register_console_command_capabilities(session, console_registry.descriptors());
}
