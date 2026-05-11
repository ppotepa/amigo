use amigo_2d_particles::Particle2dSceneService;

use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;
use crate::scheduling::AppSchedulingService;

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
        let particles = match ctx.required::<Particle2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let scheduling = match ctx.required::<AppSchedulingService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "particles.list" => {
                let emitters = particles.emitters();
                let lines = emitters
                    .iter()
                    .map(|cmd| {
                        format!(
                            "{} active={} max={} rate={} z={}",
                            cmd.entity_name,
                            particles.is_active(&cmd.entity_name),
                            cmd.emitter.max_particles,
                            cmd.emitter.spawn_rate,
                            cmd.emitter.z_index
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(if lines.is_empty() {
                    "particle emitters: none".to_owned()
                } else {
                    format!("particle emitters:\n{}", lines.join("\n"))
                })
            }
            "particles.pause" => {
                for emitter in particles.emitters() {
                    particles.set_active(&emitter.entity_name, false);
                }
                ConsoleCommandResult::ok("particles paused")
            }
            "particles.resume" => {
                for emitter in particles.emitters() {
                    particles.set_active(&emitter.entity_name, true);
                }
                ConsoleCommandResult::ok("particles resumed")
            }
            "particles.count" => {
                let count = particles.draw_commands().len();
                ConsoleCommandResult::ok(format!("particle draw commands: {count}"))
            }
            "particles.emitters" => {
                let lines = particles
                    .emitters()
                    .into_iter()
                    .map(|emitter| {
                        let quality_scale = particles.quality_scale(&emitter.entity_name);
                        let effective_max_particles = particles
                            .effective_max_particles(&emitter.entity_name)
                            .unwrap_or(emitter.emitter.max_particles);
                        let effective_spawn_rate = particles
                            .effective_spawn_rate(&emitter.entity_name)
                            .unwrap_or(emitter.emitter.spawn_rate);
                        format!(
                            "{} live={} quality_scale={:.2} effective_max_particles={} effective_spawn_rate={:.2}",
                            emitter.entity_name,
                            particles.particle_count(&emitter.entity_name),
                            quality_scale,
                            effective_max_particles,
                            effective_spawn_rate
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(if lines.is_empty() {
                    "particle emitters: none".to_owned()
                } else {
                    format!("particle emitters:\n{}", lines.join("\n"))
                })
            }
            "particles.budget" => {
                let Some(raw) = command.args.first() else {
                    return ConsoleCommandResult::ok(format!(
                        "particle budget scale={:.2}",
                        scheduling.particle_budget_scale()
                    ));
                };
                let Ok(scale) = raw.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid budget scale `{raw}`"));
                };
                scheduling.set_particle_budget_scale(scale);
                ConsoleCommandResult::ok(format!(
                    "particle budget scale set to {:.2}",
                    scheduling.particle_budget_scale()
                ))
            }
            "particles.emitter" => handle_particle_emitter_command(particles.as_ref(), &command),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn handle_particle_emitter_command(
    particles: &Particle2dSceneService,
    command: &ParsedConsoleCommand,
) -> ConsoleCommandResult {
    let [entity, op, rest @ ..] = command.args.as_slice() else {
        return ConsoleCommandResult::error(
            "usage: particles.emitter <entity> on|off|max|rate|intensity <value>",
        );
    };

    match op.as_str() {
        "on" => set_active(particles, entity, true),
        "off" => set_active(particles, entity, false),
        "max" => {
            let Some(raw) = rest.first() else {
                return ConsoleCommandResult::error(
                    "usage: particles.emitter <entity> max <count>",
                );
            };
            let Ok(count) = raw.parse::<usize>() else {
                return ConsoleCommandResult::error(format!("invalid max particle count `{raw}`"));
            };
            if particles.set_max_particles(entity, count) {
                ConsoleCommandResult::ok(format!("particle emitter `{entity}` max={count}"))
            } else {
                ConsoleCommandResult::error(format!("unknown particle emitter `{entity}`"))
            }
        }
        "rate" => {
            let Some(raw) = rest.first() else {
                return ConsoleCommandResult::error(
                    "usage: particles.emitter <entity> rate <value>",
                );
            };
            let Ok(rate) = raw.parse::<f32>() else {
                return ConsoleCommandResult::error(format!("invalid spawn rate `{raw}`"));
            };
            if particles.set_spawn_rate(entity, rate) {
                ConsoleCommandResult::ok(format!("particle emitter `{entity}` rate={rate}"))
            } else {
                ConsoleCommandResult::error(format!("unknown particle emitter `{entity}`"))
            }
        }
        "intensity" => {
            let Some(raw) = rest.first() else {
                return ConsoleCommandResult::error(
                    "usage: particles.emitter <entity> intensity <0..1>",
                );
            };
            let Ok(value) = raw.parse::<f32>() else {
                return ConsoleCommandResult::error(format!("invalid intensity `{raw}`"));
            };
            if particles.set_intensity(entity, value) {
                ConsoleCommandResult::ok(format!("particle emitter `{entity}` intensity={value}"))
            } else {
                ConsoleCommandResult::error(format!("unknown particle emitter `{entity}`"))
            }
        }
        _ => ConsoleCommandResult::error(
            "usage: particles.emitter <entity> on|off|max|rate|intensity <value>",
        ),
    }
}

fn set_active(
    particles: &Particle2dSceneService,
    entity: &str,
    active: bool,
) -> ConsoleCommandResult {
    if particles.set_active(entity, active) {
        ConsoleCommandResult::ok(format!(
            "particle emitter `{entity}` {}",
            if active { "enabled" } else { "disabled" }
        ))
    } else {
        ConsoleCommandResult::error(format!("unknown particle emitter `{entity}`"))
    }
}
