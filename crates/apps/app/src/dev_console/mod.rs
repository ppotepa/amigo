pub(crate) mod commands;
pub(crate) mod dispatcher;
pub(crate) mod model;
pub(crate) mod overlay;
pub(crate) mod parser;
pub(crate) mod registry;
pub(crate) mod theme;

use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use registry::ConsoleCommandRegistry;

pub(crate) struct DevConsoleRuntimePlugin;

impl RuntimePlugin for DevConsoleRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-dev-console"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let console_registry = ConsoleCommandRegistry::default();
        commands::register_builtin_console_commands(&console_registry);
        registry.register(console_registry)
    }
}

#[cfg(test)]
mod tests {
    use super::commands::register_builtin_console_commands;
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
}
