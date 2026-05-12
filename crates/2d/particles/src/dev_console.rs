use amigo_session::AppSchedulingService;

use crate::Particle2dSceneService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticlesDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct ParticlesDevConsoleCommandContext<'a> {
    pub particle2d_scene_service: &'a Particle2dSceneService,
    pub app_scheduling_service: &'a AppSchedulingService,
}

pub fn handle_particles_dev_console_command(
    ctx: ParticlesDevConsoleCommandContext<'_>,
    name: &str,
    args: &[String],
) -> ParticlesDevConsoleCommandOutcome {
    match name {
        "particles.list" => {
            let emitters = ctx.particle2d_scene_service.emitters();
            let lines = emitters
                .iter()
                .map(|cmd| {
                    format!(
                        "{} active={} max={} rate={} z={}",
                        cmd.entity_name,
                        ctx.particle2d_scene_service.is_active(&cmd.entity_name),
                        cmd.emitter.max_particles,
                        cmd.emitter.spawn_rate,
                        cmd.emitter.z_index
                    )
                })
                .collect::<Vec<_>>();
            ParticlesDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "particle emitters: none".to_owned()
            } else {
                format!("particle emitters:\n{}", lines.join("\n"))
            })
        }
        "particles.pause" => {
            for emitter in ctx.particle2d_scene_service.emitters() {
                ctx.particle2d_scene_service
                    .set_active(&emitter.entity_name, false);
            }
            ParticlesDevConsoleCommandOutcome::Handled("particles paused".to_owned())
        }
        "particles.resume" => {
            for emitter in ctx.particle2d_scene_service.emitters() {
                ctx.particle2d_scene_service
                    .set_active(&emitter.entity_name, true);
            }
            ParticlesDevConsoleCommandOutcome::Handled("particles resumed".to_owned())
        }
        "particles.count" => ParticlesDevConsoleCommandOutcome::Handled(format!(
            "particle draw commands: {}",
            ctx.particle2d_scene_service.draw_commands().len()
        )),
        "particles.emitters" => {
            let lines = ctx
                .particle2d_scene_service
                .emitters()
                .into_iter()
                .map(|emitter| {
                    let quality_scale = ctx
                        .particle2d_scene_service
                        .quality_scale(&emitter.entity_name);
                    let effective_max_particles = ctx
                        .particle2d_scene_service
                        .effective_max_particles(&emitter.entity_name)
                        .unwrap_or(emitter.emitter.max_particles);
                    let effective_spawn_rate = ctx
                        .particle2d_scene_service
                        .effective_spawn_rate(&emitter.entity_name)
                        .unwrap_or(emitter.emitter.spawn_rate);
                    format!(
                        "{} live={} quality_scale={:.2} effective_max_particles={} effective_spawn_rate={:.2}",
                        emitter.entity_name,
                        ctx.particle2d_scene_service
                            .particle_count(&emitter.entity_name),
                        quality_scale,
                        effective_max_particles,
                        effective_spawn_rate
                    )
                })
                .collect::<Vec<_>>();
            ParticlesDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "particle emitters: none".to_owned()
            } else {
                format!("particle emitters:\n{}", lines.join("\n"))
            })
        }
        "particles.budget" => {
            let Some(raw) = args.first() else {
                return ParticlesDevConsoleCommandOutcome::Handled(format!(
                    "particle budget scale={:.2}",
                    ctx.app_scheduling_service.particle_budget_scale()
                ));
            };
            let Ok(scale) = raw.parse::<f32>() else {
                return ParticlesDevConsoleCommandOutcome::Error(format!(
                    "invalid budget scale `{raw}`"
                ));
            };
            ctx.app_scheduling_service.set_particle_budget_scale(scale);
            ParticlesDevConsoleCommandOutcome::Handled(format!(
                "particle budget scale set to {:.2}",
                ctx.app_scheduling_service.particle_budget_scale()
            ))
        }
        "particles.emitter" => handle_particle_emitter_command(ctx.particle2d_scene_service, args),
        _ => ParticlesDevConsoleCommandOutcome::Unhandled,
    }
}

fn handle_particle_emitter_command(
    particles: &Particle2dSceneService,
    args: &[String],
) -> ParticlesDevConsoleCommandOutcome {
    let [entity, op, rest @ ..] = args else {
        return ParticlesDevConsoleCommandOutcome::Error(
            "usage: particles.emitter <entity> on|off|max|rate|intensity <value>".to_owned(),
        );
    };
    match op.as_str() {
        "on" => set_active(particles, entity, true),
        "off" => set_active(particles, entity, false),
        "max" => {
            let Some(raw) = rest.first() else {
                return ParticlesDevConsoleCommandOutcome::Error(
                    "usage: particles.emitter <entity> max <count>".to_owned(),
                );
            };
            let Ok(count) = raw.parse::<usize>() else {
                return ParticlesDevConsoleCommandOutcome::Error(format!(
                    "invalid max particle count `{raw}`"
                ));
            };
            if particles.set_max_particles(entity, count) {
                ParticlesDevConsoleCommandOutcome::Handled(format!(
                    "particle emitter `{entity}` max={count}"
                ))
            } else {
                ParticlesDevConsoleCommandOutcome::Error(format!(
                    "unknown particle emitter `{entity}`"
                ))
            }
        }
        "rate" => {
            let Some(raw) = rest.first() else {
                return ParticlesDevConsoleCommandOutcome::Error(
                    "usage: particles.emitter <entity> rate <value>".to_owned(),
                );
            };
            let Ok(rate) = raw.parse::<f32>() else {
                return ParticlesDevConsoleCommandOutcome::Error(format!(
                    "invalid spawn rate `{raw}`"
                ));
            };
            if particles.set_spawn_rate(entity, rate) {
                ParticlesDevConsoleCommandOutcome::Handled(format!(
                    "particle emitter `{entity}` rate={rate}"
                ))
            } else {
                ParticlesDevConsoleCommandOutcome::Error(format!(
                    "unknown particle emitter `{entity}`"
                ))
            }
        }
        "intensity" => {
            let Some(raw) = rest.first() else {
                return ParticlesDevConsoleCommandOutcome::Error(
                    "usage: particles.emitter <entity> intensity <0..1>".to_owned(),
                );
            };
            let Ok(value) = raw.parse::<f32>() else {
                return ParticlesDevConsoleCommandOutcome::Error(format!(
                    "invalid intensity `{raw}`"
                ));
            };
            if particles.set_intensity(entity, value) {
                ParticlesDevConsoleCommandOutcome::Handled(format!(
                    "particle emitter `{entity}` intensity={value}"
                ))
            } else {
                ParticlesDevConsoleCommandOutcome::Error(format!(
                    "unknown particle emitter `{entity}`"
                ))
            }
        }
        _ => ParticlesDevConsoleCommandOutcome::Error(
            "usage: particles.emitter <entity> on|off|max|rate|intensity <value>".to_owned(),
        ),
    }
}

fn set_active(
    particles: &Particle2dSceneService,
    entity: &str,
    active: bool,
) -> ParticlesDevConsoleCommandOutcome {
    if particles.set_active(entity, active) {
        ParticlesDevConsoleCommandOutcome::Handled(format!(
            "particle emitter `{entity}` {}",
            if active { "enabled" } else { "disabled" }
        ))
    } else {
        ParticlesDevConsoleCommandOutcome::Error(format!("unknown particle emitter `{entity}`"))
    }
}
