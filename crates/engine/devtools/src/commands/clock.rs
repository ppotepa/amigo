use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};
use amigo_session::{
    ResolvedFrameClockStrategy, ResolvedPresentationLayerMode, RuntimeFrameClockService,
};

pub(crate) struct ClockConsoleCommandHandler;

impl ConsoleCommandHandler for ClockConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "clock-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "clock.stats",
                aliases: &[],
                category: "runtime",
                help: "Show host, simulation, and sampled game render frame clock stats.",
                usage: "clock.stats",
                examples: &["clock.stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "clock.set",
                aliases: &[],
                category: "runtime",
                help: "Set frame clock strategy, fps, catch-up, dt clamp, or cache presentation.",
                usage: "clock.set <strategy|render_fps|simulation_fps|max_catch_up_ticks|clamp_dt|cache> <value>",
                examples: &[
                    "clock.set strategy fixed_simulation_sampled_render",
                    "clock.set render_fps 12",
                    "clock.set simulation_fps 60",
                    "clock.set max_catch_up_ticks 1",
                    "clock.set clamp_dt 0.05",
                    "clock.set cache true",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "clock.cache",
                aliases: &[],
                category: "runtime",
                help: "Manage cached game frame state.",
                usage: "clock.cache invalidate",
                examples: &["clock.cache invalidate"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name.starts_with("clock.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let clock = match ctx.required::<RuntimeFrameClockService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "clock.stats" => {
                let snapshot = clock.snapshot();
                ConsoleCommandResult::ok(format!(
                    "clock:\n  strategy={}\n  host_fps={:.1} host_dt={:.4}\n  simulation_fps={:.1} target={:.1} dt={:.4} tick={}\n  game_render_fps={:.1} target={:.1} frame={}\n  scheduled_sim_ticks={}\n  consumed_sim_ticks={}\n  pending_sim_ticks={}\n  dropped_sim_debt={:.4}\n  render_due={}\n  holding_cached_frame={}",
                    strategy_label(snapshot.strategy),
                    snapshot.actual_host_fps,
                    snapshot.host_delta_seconds,
                    if snapshot.simulation_delta_seconds > 0.0 {
                        1.0 / snapshot.simulation_delta_seconds
                    } else {
                        0.0
                    },
                    snapshot.target_simulation_fps,
                    snapshot.simulation_delta_seconds,
                    snapshot.simulation_tick_index,
                    snapshot.actual_game_render_fps,
                    snapshot.target_render_fps,
                    snapshot.game_render_frame_index,
                    snapshot.scheduled_simulation_ticks,
                    snapshot.consumed_simulation_ticks,
                    snapshot.pending_simulation_ticks,
                    snapshot.dropped_simulation_debt_seconds,
                    snapshot.should_render_game_frame,
                    snapshot.holding_cached_game_frame
                ))
            }
            "clock.set" => handle_clock_set(clock.as_ref(), command.args.as_slice()),
            "clock.cache" => handle_clock_cache(clock.as_ref(), command.args.as_slice()),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn handle_clock_set(clock: &RuntimeFrameClockService, args: &[String]) -> ConsoleCommandResult {
    let Some(field) = args.first().map(String::as_str) else {
        return ConsoleCommandResult::error(
            "usage: clock.set <strategy|render_fps|simulation_fps|max_catch_up_ticks|clamp_dt|cache> <value>",
        );
    };
    let Some(value) = args.get(1).map(String::as_str) else {
        return ConsoleCommandResult::error(format!("missing value for `{field}`"));
    };

    let mut config = clock.config();
    match field {
        "strategy" => {
            let Some(strategy) = parse_strategy(value) else {
                return ConsoleCommandResult::error(format!(
                    "unsupported strategy `{value}`; expected host_realtime|fixed_update_and_render|fixed_simulation_sampled_render|realtime_update_sampled_render"
                ));
            };
            config.strategy = strategy;
        }
        "render_fps" => {
            let Some(fps) = parse_fps(value) else {
                return ConsoleCommandResult::error(
                    "render_fps must be a finite value from 1 to 240",
                );
            };
            config.render_fps = fps;
        }
        "simulation_fps" => {
            let Some(fps) = parse_fps(value) else {
                return ConsoleCommandResult::error(
                    "simulation_fps must be a finite value from 1 to 240",
                );
            };
            config.simulation_fps = fps;
        }
        "max_catch_up_ticks" => {
            let Ok(max) = value.parse::<u32>() else {
                return ConsoleCommandResult::error(
                    "max_catch_up_ticks must be an integer from 1 to 30",
                );
            };
            config.max_catch_up_ticks = max.clamp(1, 30);
        }
        "clamp_frame_delta_seconds" | "clamp_dt" => {
            let Some(seconds) = parse_positive_seconds(value) else {
                return ConsoleCommandResult::error(
                    "clamp_frame_delta_seconds must be finite and > 0",
                );
            };
            config.clamp_frame_delta_seconds = seconds.clamp(0.016, 1.0);
        }
        "cache" => {
            let Some(enabled) = parse_bool(value) else {
                return ConsoleCommandResult::error("cache must be true or false");
            };
            config.presentation.cache_game_frame = enabled;
            config.presentation.hold_last_game_frame = enabled;
        }
        "game_ui" => {
            config.presentation.game_ui = match value {
                "cached" => ResolvedPresentationLayerMode::Cached,
                "live" => ResolvedPresentationLayerMode::Live,
                _ => return ConsoleCommandResult::error("game_ui must be cached or live"),
            };
        }
        _ => {
            return ConsoleCommandResult::error(
                "usage: clock.set <strategy|render_fps|simulation_fps|max_catch_up_ticks|clamp_dt|cache|game_ui> <value>",
            );
        }
    }

    clock.configure(config);
    ConsoleCommandResult::ok(format!("clock {field}={value}"))
}

fn handle_clock_cache(clock: &RuntimeFrameClockService, args: &[String]) -> ConsoleCommandResult {
    match args.first().map(String::as_str) {
        Some("invalidate") => {
            clock.mark_game_frame_cache_invalid();
            ConsoleCommandResult::ok("clock cache invalidated")
        }
        _ => ConsoleCommandResult::error("usage: clock.cache invalidate"),
    }
}

fn parse_strategy(value: &str) -> Option<ResolvedFrameClockStrategy> {
    match value {
        "host_realtime" => Some(ResolvedFrameClockStrategy::HostRealtime),
        "fixed_update_and_render" => Some(ResolvedFrameClockStrategy::FixedUpdateAndRender),
        "fixed_simulation_sampled_render" => {
            Some(ResolvedFrameClockStrategy::FixedSimulationSampledRender)
        }
        "realtime_update_sampled_render" => {
            Some(ResolvedFrameClockStrategy::RealtimeUpdateSampledRender)
        }
        _ => None,
    }
}

fn parse_fps(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|fps| fps.is_finite() && *fps > 0.0)
        .map(|fps| fps.clamp(1.0, 240.0))
}

fn parse_positive_seconds(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "on" | "1" => Some(true),
        "false" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn strategy_label(strategy: ResolvedFrameClockStrategy) -> &'static str {
    match strategy {
        ResolvedFrameClockStrategy::HostRealtime => "host_realtime",
        ResolvedFrameClockStrategy::FixedUpdateAndRender => "fixed_update_and_render",
        ResolvedFrameClockStrategy::FixedSimulationSampledRender => {
            "fixed_simulation_sampled_render"
        }
        ResolvedFrameClockStrategy::RealtimeUpdateSampledRender => "realtime_update_sampled_render",
    }
}
