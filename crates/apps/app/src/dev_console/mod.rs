pub(crate) mod commands;
pub(crate) mod completion;
pub(crate) mod dispatcher;
pub(crate) mod model;
pub(crate) mod overlay;
pub(crate) mod parser;
pub(crate) mod registry;
pub(crate) mod theme;

use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityKind, RuntimeCapabilityDescriptor, RuntimeCapability,
        RuntimeDomainId, APP_HOST_DOMAIN_ID,
    },
    RuntimeSession,
};

use registry::ConsoleCommandRegistry;

pub(crate) struct DevConsoleRuntimePlugin;

pub(crate) struct AppDevConsoleCommandProvider;

impl AppDevConsoleCommandProvider {
    pub(crate) fn register_dev_console_commands(&self, session: &mut RuntimeSession) {
        let console_registry = ConsoleCommandRegistry::default();
        commands::register_builtin_console_commands(&console_registry);

        for descriptor in console_registry.descriptors().into_iter() {
            if matches!(
                descriptor.category,
                "scene"
                    | "assets"
                    | "particles"
                    | "layered-image"
                    | "lighting"
                    | "composition"
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
                        domain_id: RuntimeDomainId::new(if is_host_category {
                            APP_HOST_DOMAIN_ID
                        } else {
                            APP_HOST_DOMAIN_ID
                        }),
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
}

pub(crate) fn register_app_dev_console_command_provider(
    session: &mut RuntimeSession,
) {
    let provider = AppDevConsoleCommandProvider;
    provider.register_dev_console_commands(session)
}

impl RuntimePlugin for DevConsoleRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-dev-console"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let console_registry = ConsoleCommandRegistry::default();
        commands::register_builtin_console_commands(&console_registry);
        registry.register(console_registry)?;
        registry.register(completion::ConsoleCompletionState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::commands::register_builtin_console_commands;
    use super::completion::compute_console_completion;
    use super::parser::parse_console_command;
    use super::registry::ConsoleCommandRegistry;

    #[test]
    fn registry_finds_core_help_handler() {
        let registry = ConsoleCommandRegistry::default();
        register_builtin_console_commands(&registry);
        let parsed = parse_console_command("help").unwrap();
        assert!(registry.handler_for(&parsed).is_some());
    }

    #[test]
    fn registry_finds_render_and_particle_handlers() {
        let registry = ConsoleCommandRegistry::default();
        register_builtin_console_commands(&registry);
        assert!(
            registry
                .handler_for(&parse_console_command("render.stats").unwrap())
                .is_some()
        );
        assert!(
            registry
                .handler_for(&parse_console_command("particles.list").unwrap())
                .is_some()
        );
    }

    #[test]
    fn registry_finds_debug_overlay_handlers() {
        let registry = ConsoleCommandRegistry::default();
        register_builtin_console_commands(&registry);

        for command in [
            "debug.overlay",
            "debug.fps",
            "debug.fps_graph",
            "debug.graphs",
            "debug.stats",
            "debug.particles",
            "debug.render",
            "debug.audio",
            "debug.input",
            "debug.lights",
            "debug.layers",
            "debug.timings",
            "debug.scheduler",
            "debug.memory",
            "debug.dump",
            "debug.reset",
        ] {
            assert!(
                registry
                    .handler_for(&parse_console_command(command).unwrap())
                    .is_some(),
                "missing handler for {command}"
            );
        }
    }

    #[test]
    fn completion_suggests_registered_debug_commands() {
        let registry = ConsoleCommandRegistry::default();
        register_builtin_console_commands(&registry);

        let completion = compute_console_completion("debug.fp", &registry)
            .expect("completion should be available");

        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "debug.fps")
        );
        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "debug.fps_graph")
        );
    }

    #[test]
    fn completion_suggests_toggle_arguments_from_usage() {
        let registry = ConsoleCommandRegistry::default();
        register_builtin_console_commands(&registry);

        let completion = compute_console_completion("debug.fps o", &registry)
            .expect("completion should be available");

        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "on")
        );
        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "off")
        );
    }
}
