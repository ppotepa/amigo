use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

pub(crate) struct ParticlesConsoleCommandHandler;

impl ConsoleCommandHandler for ParticlesConsoleCommandHandler {
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
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let particles = match ctx.required::<amigo_particles_2d_plugin::Particle2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let scheduling = match ctx.required::<amigo_session::RuntimeSchedulingService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match amigo_particles_2d_plugin::handle_particles_dev_console_command(
            amigo_particles_2d_plugin::ParticlesDevConsoleCommandContext {
                particle2d_scene_service: particles.as_ref(),
                app_scheduling_service: scheduling.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            amigo_particles_2d_plugin::ParticlesDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            amigo_particles_2d_plugin::ParticlesDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            amigo_particles_2d_plugin::ParticlesDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}
