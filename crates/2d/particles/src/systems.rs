use amigo_core::{AmigoError, AmigoResult};
use amigo_math::{Transform2, Vec2};
use amigo_runtime::{
    EngineJob, EngineLane, EngineSchedulerMode, EngineSchedulingConfig, EngineTaskSystem,
    JobContext, Runtime,
};
use amigo_scene::SceneService;
use amigo_2d_motion::Motion2dSceneService;
use amigo_session::{AppSchedulingService, SchedulingOverrideReport};
use std::time::Instant;

use crate::{Particle2dEmitterRuntimeInput, Particle2dFrameJobResult, Particle2dSceneService};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

struct ParticleTickJob {
    service: std::sync::Arc<Particle2dSceneService>,
    inputs: Vec<Particle2dEmitterRuntimeInput>,
    delta_seconds: f32,
}

impl EngineJob for ParticleTickJob {
    type Output = Particle2dFrameJobResult;

    fn name(&self) -> &'static str {
        "particles_2d_frame"
    }

    fn lane(&self) -> EngineLane {
        EngineLane::Simulation
    }

    fn run(self, _ctx: JobContext) -> Self::Output {
        self.service
            .tick_inline_job_equivalent(&self.inputs, self.delta_seconds)
    }
}

struct ParticleJobCompletionGuard {
    scheduling: std::sync::Arc<AppSchedulingService>,
}

impl ParticleJobCompletionGuard {
    fn new(scheduling: std::sync::Arc<AppSchedulingService>) -> Self {
        Self { scheduling }
    }
}

impl Drop for ParticleJobCompletionGuard {
    fn drop(&mut self) {
        self.scheduling.finish_particle_job();
    }
}

#[derive(Debug, Clone)]
struct ParticleSchedulingTargetResolution {
    matched_entity_name: Option<String>,
    reason: Option<String>,
}

impl ParticleSchedulingTargetResolution {
    fn matched(entity_name: String) -> Self {
        Self {
            matched_entity_name: Some(entity_name),
            reason: None,
        }
    }

    fn unmatched(reason: String) -> Self {
        Self {
            matched_entity_name: None,
            reason: Some(reason),
        }
    }

    fn not_particle_target(reason: &str) -> Self {
        Self {
            matched_entity_name: None,
            reason: Some(reason.to_owned()),
        }
    }
}

pub fn tick_particles_2d_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let motion_scene_service = required::<Motion2dSceneService>(runtime)?;
    let particle_scene_service = required::<Particle2dSceneService>(runtime)?;
    let scheduling = required::<AppSchedulingService>(runtime)?;
    let task_system = required::<EngineTaskSystem>(runtime)?;

    let emitter_commands = particle_scene_service.emitters();
    let emitter_names = emitter_commands
        .iter()
        .map(|command| command.entity_name.as_str())
        .collect::<Vec<_>>();

    let inputs = emitter_commands
        .iter()
        .filter_map(|command| {
            let source_name = command
                .emitter
                .attached_to
                .as_deref()
                .unwrap_or(command.entity_name.as_str());
            let source_transform = scene_service.transform_of(source_name)?;
            Some(Particle2dEmitterRuntimeInput {
                emitter_entity_name: command.entity_name.clone(),
                source_entity_name: source_name.to_owned(),
                source_transform: Transform2 {
                    translation: Vec2::new(
                        source_transform.translation.x,
                        source_transform.translation.y,
                    ),
                    rotation_radians: source_transform.rotation_euler.z,
                    scale: Vec2::new(source_transform.scale.x, source_transform.scale.y),
                },
                source_velocity: motion_scene_service.current_velocity(source_name),
                source_visible: scene_service.is_visible(source_name),
                source_simulation_enabled: scene_service.is_simulation_enabled(source_name),
            })
        })
        .collect::<Vec<_>>();

    let scheduling_config = scheduling.config();
    let global_budget_scale = scheduling.particle_budget_scale();
    let mode = scheduling_config.mode;
    particle_scene_service.clear_quality_scales();
    let mut override_reports = Vec::new();
    if mode != EngineSchedulerMode::SingleThread {
        for override_entry in &scheduling_config.overrides {
            let Some(scale) = override_entry.quality_scale else {
                continue;
            };
            let effective_scale = (scale * global_budget_scale).clamp(0.0, 1.0);
            let resolution = resolve_particle_emitter_target(
                &override_entry.target,
                emitter_names.iter().copied(),
            );
            if let Some(entity_name) = resolution.matched_entity_name.as_deref() {
                let matched =
                    particle_scene_service.set_quality_scale(entity_name, effective_scale);
                override_reports.push(SchedulingOverrideReport {
                    target: override_entry.target.clone(),
                    domain: "particles2d".to_owned(),
                    matched,
                    resolved_target: Some(entity_name.to_owned()),
                    quality_scale: Some(effective_scale),
                    reason: if matched {
                        None
                    } else {
                        Some("set_quality_scale rejected target".to_owned())
                    },
                });
            } else {
                override_reports.push(SchedulingOverrideReport {
                    target: override_entry.target.clone(),
                    domain: "particles2d".to_owned(),
                    matched: false,
                    resolved_target: None,
                    quality_scale: Some(effective_scale),
                    reason: resolution.reason,
                });
            }
        }
    }
    scheduling.set_override_reports(override_reports);

    task_system.set_config(EngineSchedulingConfig {
        mode,
        max_workers: scheduling_config.max_workers,
        deterministic: scheduling_config.deterministic,
        allow_frame_latency: scheduling_config.allow_frame_latency,
    });

    let start = Instant::now();
    let mut worker_jobs_submitted = 0usize;
    let mut worker_jobs_completed = 0usize;
    let mut particle_live_count = 0usize;
    let mut particle_spawned_count = 0usize;
    let mut worker_waited_this_frame = false;
    let mut reused_previous_particle_frame = false;
    let particle_mode: String;

    match mode {
        EngineSchedulerMode::SingleThread => {
            let result = particle_scene_service.tick(&inputs, delta_seconds);
            particle_live_count = result.stats.live_particles;
            particle_spawned_count = result.stats.spawned_particles;
            particle_mode = "legacy".to_owned();
        }
        EngineSchedulerMode::Auto => {
            let result = particle_scene_service.tick_inline_job_equivalent(&inputs, delta_seconds);
            particle_live_count = result.stats.live_particles;
            particle_spawned_count = result.stats.spawned_particles;
            particle_mode = "inline_job".to_owned();
        }
        EngineSchedulerMode::Hybrid | EngineSchedulerMode::Manual => {
            let latency_allowed = scheduling_config.allow_frame_latency
                || scheduling_config.overrides.iter().any(|entry| {
                    entry.target == "render:particles2d"
                        && entry.allow_frame_latency.unwrap_or(false)
                });
            if latency_allowed && scheduling_config.max_workers > 0 {
                if scheduling.try_begin_particle_job() {
                    let scheduling_service = scheduling.clone();
                    let particle_service = particle_scene_service.clone();
                    let inputs_for_job = inputs.clone();
                    let delta_for_job = delta_seconds;
                    let submitted = task_system.spawn_detached(move || {
                        let _guard = ParticleJobCompletionGuard::new(scheduling_service);
                        let _ = particle_service
                            .tick_inline_job_equivalent(&inputs_for_job, delta_for_job);
                    });
                    if submitted {
                        worker_jobs_submitted = 1;
                        reused_previous_particle_frame = true;
                        particle_mode = "async_worker_cached".to_owned();
                    } else {
                        scheduling.finish_particle_job();
                        let result = particle_scene_service
                            .tick_inline_job_equivalent(&inputs, delta_seconds);
                        particle_live_count = result.stats.live_particles;
                        particle_spawned_count = result.stats.spawned_particles;
                        particle_mode = "inline_fallback".to_owned();
                    }
                } else {
                    reused_previous_particle_frame = true;
                    particle_mode = "async_worker_in_flight_reuse".to_owned();
                }
            } else if scheduling_config.max_workers > 0 {
                worker_jobs_submitted = 1;
                worker_waited_this_frame = true;
                let result = task_system.run(
                    ParticleTickJob {
                        service: particle_scene_service.clone(),
                        inputs: inputs.clone(),
                        delta_seconds,
                    },
                    JobContext {
                        frame_index: 0,
                        lane: EngineLane::Simulation,
                    },
                );
                worker_jobs_completed = 1;
                particle_live_count = result.stats.live_particles;
                particle_spawned_count = result.stats.spawned_particles;
                particle_mode = "worker_job_blocking".to_owned();
            } else {
                let result =
                    particle_scene_service.tick_inline_job_equivalent(&inputs, delta_seconds);
                particle_live_count = result.stats.live_particles;
                particle_spawned_count = result.stats.spawned_particles;
                particle_mode = "inline_job".to_owned();
            }
        }
    }

    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    scheduling.set_stats(amigo_session::SchedulingFrameStats {
        mode,
        particle_mode,
        particle_update_ms: elapsed_ms,
        render_prepare_ms: 0.0,
        worker_jobs_submitted,
        worker_jobs_completed,
        particle_live_count,
        particle_spawned_count,
        worker_waited_this_frame,
        particle_job_in_flight: scheduling.particle_job_in_flight(),
        reused_previous_particle_frame,
    });

    Ok(())
}

fn particle_emitter_target_entity(target: &str) -> Option<&str> {
    let (entity_part, component_part) = target.split_once("/component:")?;
    if component_part != "ParticleEmitter2D" {
        return None;
    }
    entity_part.strip_prefix("entity:")
}

fn resolve_particle_emitter_target<'a>(
    target: &str,
    emitter_names: impl IntoIterator<Item = &'a str>,
) -> ParticleSchedulingTargetResolution {
    let Some(raw_entity) = particle_emitter_target_entity(target) else {
        return ParticleSchedulingTargetResolution::not_particle_target(
            "target is not entity:<name>/component:ParticleEmitter2D",
        );
    };

    let names = emitter_names.into_iter().collect::<Vec<_>>();
    if names.iter().any(|name| *name == raw_entity) {
        return ParticleSchedulingTargetResolution::matched(raw_entity.to_owned());
    }

    let suffix = format!("-{raw_entity}");
    let suffix_matches = names
        .iter()
        .filter(|name| name.ends_with(&suffix))
        .copied()
        .collect::<Vec<_>>();

    match suffix_matches.as_slice() {
        [single] => ParticleSchedulingTargetResolution::matched((*single).to_owned()),
        [] => ParticleSchedulingTargetResolution::unmatched(format!(
            "no particle emitter matched `{raw_entity}`"
        )),
        many => ParticleSchedulingTargetResolution::unmatched(format!(
            "ambiguous particle emitter target `{raw_entity}` matched {} emitters",
            many.len()
        )),
    }
}

