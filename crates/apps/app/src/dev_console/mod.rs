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

use registry::ConsoleCommandRegistry;

pub(crate) struct DevConsoleRuntimePlugin;

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
