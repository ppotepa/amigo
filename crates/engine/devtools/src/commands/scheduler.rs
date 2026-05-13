use amigo_runtime::EngineSchedulerMode;

use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use amigo_session::AppSchedulingService;

pub(crate) struct SchedulerConsoleCommandHandler;

impl ConsoleCommandHandler for SchedulerConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "scheduler-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "scheduler.stats",
                aliases: &[],
                category: "runtime",
                help: "Show current scheduler stats.",
                usage: "scheduler.stats",
                examples: &["scheduler.stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scheduler.mode",
                aliases: &[],
                category: "runtime",
                help: "Show current scheduler mode.",
                usage: "scheduler.mode",
                examples: &["scheduler.mode"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scheduler.overrides",
                aliases: &[],
                category: "runtime",
                help: "Show resolved scheduling override diagnostics.",
                usage: "scheduler.overrides",
                examples: &["scheduler.overrides"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scheduler.set",
                aliases: &[],
                category: "runtime",
                help: "Set scheduler mode: single_thread|auto|hybrid|manual.",
                usage: "scheduler.set <single_thread|auto|hybrid|manual>",
                examples: &["scheduler.set single_thread", "scheduler.set hybrid"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name.starts_with("scheduler.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let scheduling = match ctx.required::<AppSchedulingService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "scheduler.stats" => {
                let stats = scheduling.stats();
                ConsoleCommandResult::ok(format!(
                    "scheduler={} particle_mode={} particle_update_ms={:.3} render_prepare_ms={:.3} worker_jobs_submitted={} worker_jobs_completed={} live_particles={} spawned={} waited={} in_flight={} reused_previous={}",
                    mode_label(stats.mode),
                    stats.particle_mode,
                    stats.particle_update_ms,
                    stats.render_prepare_ms,
                    stats.worker_jobs_submitted,
                    stats.worker_jobs_completed,
                    stats.particle_live_count,
                    stats.particle_spawned_count,
                    stats.worker_waited_this_frame,
                    stats.particle_job_in_flight,
                    stats.reused_previous_particle_frame
                ))
            }
            "scheduler.overrides" => {
                let reports = scheduling.override_reports();
                if reports.is_empty() {
                    return ConsoleCommandResult::ok("scheduler overrides: none reported yet");
                }
                let lines = reports
                    .into_iter()
                    .map(|report| {
                        format!(
                            "{} domain={} matched={} resolved={} quality_scale={} reason={}",
                            report.target,
                            report.domain,
                            report.matched,
                            report.resolved_target.unwrap_or_else(|| "-".to_owned()),
                            report
                                .quality_scale
                                .map(|value| format!("{value:.2}"))
                                .unwrap_or_else(|| "-".to_owned()),
                            report.reason.unwrap_or_else(|| "-".to_owned())
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(lines.join("\n"))
            }
            "scheduler.mode" => {
                ConsoleCommandResult::ok(format!("scheduler={}", mode_label(scheduling.mode())))
            }
            "scheduler.set" => {
                let Some(mode_text) = command.args.first().map(String::as_str) else {
                    return ConsoleCommandResult::error(
                        "usage: scheduler.set <single_thread|auto|hybrid|manual>",
                    );
                };

                let Some(mode) = parse_mode(mode_text) else {
                    return ConsoleCommandResult::error(format!(
                        "unsupported mode `{mode_text}`; expected single_thread|auto|hybrid|manual"
                    ));
                };

                scheduling.set_mode(mode);
                ConsoleCommandResult::ok(format!("scheduler={}", mode_label(mode)))
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn parse_mode(value: &str) -> Option<EngineSchedulerMode> {
    match value {
        "single_thread" => Some(EngineSchedulerMode::SingleThread),
        "auto" => Some(EngineSchedulerMode::Auto),
        "hybrid" => Some(EngineSchedulerMode::Hybrid),
        "manual" => Some(EngineSchedulerMode::Manual),
        _ => None,
    }
}

fn mode_label(mode: EngineSchedulerMode) -> &'static str {
    match mode {
        EngineSchedulerMode::SingleThread => "single_thread",
        EngineSchedulerMode::Auto => "auto",
        EngineSchedulerMode::Hybrid => "hybrid",
        EngineSchedulerMode::Manual => "manual",
    }
}



