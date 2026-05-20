use amigo_devtools::{
    ConsoleCommandDescriptor, ConsoleCommandResult, DevConsoleCommandContext,
    ParsedConsoleCommand, RuntimeConsoleCommandHandler,
};

pub struct ParticlesConsoleCommandHandler;

impl RuntimeConsoleCommandHandler for ParticlesConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "particles-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "particles.list",
                aliases: &[],
                category: "particles",
                help: "List particle emitters.",
                usage: "particles.list",
                examples: &["particles.list"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "particles.pause",
                aliases: &[],
                category: "particles",
                help: "Disable all particle emitters.",
                usage: "particles.pause",
                examples: &["particles.pause"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "particles.emitters",
                aliases: &[],
                category: "particles",
                help: "Show emitter live counts and effective budget.",
                usage: "particles.emitters",
                examples: &["particles.emitters"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "particles.budget",
                aliases: &[],
                category: "particles",
                help: "Set a temporary global particle budget multiplier.",
                usage: "particles.budget <scale>",
                examples: &["particles.budget 1.0", "particles.budget 0.5"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name.starts_with("particles.")
    }

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let particles = match ctx.required::<crate::Particle2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let scheduling = match ctx.required::<amigo_session::RuntimeSchedulingService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match crate::handle_particles_dev_console_command(
            crate::ParticlesDevConsoleCommandContext {
                particle2d_scene_service: particles.as_ref(),
                app_scheduling_service: scheduling.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            crate::ParticlesDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            crate::ParticlesDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            crate::ParticlesDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}
