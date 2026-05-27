use std::sync::Mutex;
use std::time::{Duration, Instant};

use amigo_runtime::Runtime;

use crate::{ResolvedFrameClockConfig, ResolvedFrameClockStrategy};

#[derive(Debug, Clone)]
pub struct RuntimeFrameClockSnapshot {
    pub strategy: ResolvedFrameClockStrategy,
    pub host_frame_index: u64,
    pub simulation_tick_index: u64,
    pub game_render_frame_index: u64,
    pub host_delta_seconds: f32,
    pub simulation_delta_seconds: f32,
    pub target_simulation_fps: f32,
    pub target_render_fps: f32,
    pub actual_host_fps: f32,
    pub actual_game_render_fps: f32,
    pub scheduled_simulation_ticks: u32,
    pub consumed_simulation_ticks: u32,
    pub pending_simulation_ticks: u32,
    pub dropped_simulation_debt_seconds: f32,
    pub should_render_game_frame: bool,
    pub holding_cached_game_frame: bool,
}

pub struct RuntimeFrameClockService {
    inner: Mutex<RuntimeFrameClockState>,
}

struct RuntimeFrameClockState {
    config: ResolvedFrameClockConfig,
    last_host_instant: Option<Instant>,
    host_frame_index: u64,
    simulation_tick_index: u64,
    game_render_frame_index: u64,
    simulation_accumulator: Duration,
    render_accumulator: Duration,
    host_delta_seconds: f32,
    simulation_delta_seconds: f32,
    actual_host_fps: f32,
    actual_game_render_fps: f32,
    scheduled_simulation_ticks: u32,
    consumed_simulation_ticks: u32,
    pending_simulation_ticks: u32,
    dropped_simulation_debt_seconds: f32,
    last_game_render_instant: Option<Instant>,
    render_due: bool,
    cache_valid: bool,
}

impl Default for RuntimeFrameClockService {
    fn default() -> Self {
        let config = ResolvedFrameClockConfig::default();
        Self {
            inner: Mutex::new(RuntimeFrameClockState {
                simulation_delta_seconds: 1.0 / config.simulation_fps,
                config,
                last_host_instant: None,
                host_frame_index: 0,
                simulation_tick_index: 0,
                game_render_frame_index: 0,
                simulation_accumulator: Duration::ZERO,
                render_accumulator: Duration::ZERO,
                host_delta_seconds: 1.0 / 60.0,
                actual_host_fps: 60.0,
                actual_game_render_fps: 60.0,
                scheduled_simulation_ticks: 0,
                consumed_simulation_ticks: 0,
                pending_simulation_ticks: 0,
                dropped_simulation_debt_seconds: 0.0,
                last_game_render_instant: None,
                render_due: true,
                cache_valid: false,
            }),
        }
    }
}

impl RuntimeFrameClockService {
    pub fn configure(&self, config: ResolvedFrameClockConfig) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        state.config = config;
        state.simulation_accumulator = Duration::ZERO;
        state.render_accumulator = Duration::ZERO;
        state.pending_simulation_ticks = 0;
        state.scheduled_simulation_ticks = 0;
        state.consumed_simulation_ticks = 0;
        state.dropped_simulation_debt_seconds = 0.0;
        state.last_game_render_instant = None;
        state.render_due = true;
        state.cache_valid = false;
        state.simulation_delta_seconds = 1.0 / state.config.simulation_fps;
    }

    pub fn config(&self) -> ResolvedFrameClockConfig {
        self.inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned")
            .config
            .clone()
    }

    pub fn begin_host_frame(&self, now: Instant) -> RuntimeFrameClockSnapshot {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        let previous = state.last_host_instant.replace(now);
        let raw_host_delta = previous
            .map(|instant| now.saturating_duration_since(instant))
            .unwrap_or_else(|| Duration::from_secs_f32(1.0 / state.config.simulation_fps));
        let host_delta = raw_host_delta.min(Duration::from_secs_f32(
            state.config.clamp_frame_delta_seconds,
        ));

        state.host_frame_index += 1;
        state.host_delta_seconds = host_delta.as_secs_f32();
        state.actual_host_fps = fps_from_delta(state.host_delta_seconds);
        state.consumed_simulation_ticks = 0;
        state.dropped_simulation_debt_seconds = 0.0;

        match state.config.strategy {
            ResolvedFrameClockStrategy::HostRealtime => {
                state.pending_simulation_ticks = 1;
                state.simulation_delta_seconds = state.host_delta_seconds;
                state.render_due = true;
            }
            ResolvedFrameClockStrategy::FixedUpdateAndRender => {
                let step = Duration::from_secs_f32(1.0 / state.config.simulation_fps);
                let max_catch_up_ticks = state.config.max_catch_up_ticks;
                state.simulation_accumulator += host_delta;
                state.render_accumulator += host_delta;
                let consumed =
                    consume_ticks(&mut state.simulation_accumulator, step, max_catch_up_ticks);
                state.pending_simulation_ticks = consumed.ticks;
                state.dropped_simulation_debt_seconds = consumed.dropped_debt.as_secs_f32();
                state.simulation_delta_seconds = step.as_secs_f32();
                state.render_due = state.render_accumulator >= step || !state.cache_valid;
                if state.render_due {
                    state.render_accumulator = Duration::ZERO;
                }
            }
            ResolvedFrameClockStrategy::FixedSimulationSampledRender => {
                let sim_step = Duration::from_secs_f32(1.0 / state.config.simulation_fps);
                let render_step = Duration::from_secs_f32(1.0 / state.config.render_fps);
                let max_catch_up_ticks = state.config.max_catch_up_ticks;
                state.simulation_accumulator += host_delta;
                state.render_accumulator += host_delta;
                let consumed = consume_ticks(
                    &mut state.simulation_accumulator,
                    sim_step,
                    max_catch_up_ticks,
                );
                state.pending_simulation_ticks = consumed.ticks;
                state.dropped_simulation_debt_seconds = consumed.dropped_debt.as_secs_f32();
                state.simulation_delta_seconds = sim_step.as_secs_f32();
                state.render_due = state.render_accumulator >= render_step || !state.cache_valid;
                if state.render_due {
                    state.render_accumulator = Duration::ZERO;
                }
            }
            ResolvedFrameClockStrategy::RealtimeUpdateSampledRender => {
                let render_step = Duration::from_secs_f32(1.0 / state.config.render_fps);
                state.pending_simulation_ticks = 1;
                state.simulation_delta_seconds = state.host_delta_seconds;
                state.render_accumulator += host_delta;
                state.render_due = state.render_accumulator >= render_step || !state.cache_valid;
                if state.render_due {
                    state.render_accumulator = Duration::ZERO;
                }
            }
        }
        state.scheduled_simulation_ticks = state.pending_simulation_ticks;

        snapshot_from_state(&state)
    }

    pub fn take_pending_simulation_tick_count(&self) -> (u32, f32) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        let ticks = state.pending_simulation_ticks;
        let dt = state.simulation_delta_seconds;
        state.pending_simulation_ticks = 0;
        state.consumed_simulation_ticks = ticks;
        state.simulation_tick_index += u64::from(ticks);
        (ticks, dt)
    }

    pub fn take_pending_simulation_ticks(&self) -> Vec<f32> {
        let (ticks, dt) = self.take_pending_simulation_tick_count();
        vec![dt; ticks as usize]
    }

    pub fn should_render_game_frame(&self) -> bool {
        self.inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned")
            .render_due
    }

    pub fn mark_game_frame_rendered(&self) {
        self.mark_game_frame_rendered_at(Instant::now());
    }

    pub fn mark_game_frame_rendered_at(&self, now: Instant) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        state.game_render_frame_index += 1;
        state.cache_valid = true;
        state.render_due = false;
        if let Some(previous) = state.last_game_render_instant.replace(now) {
            state.actual_game_render_fps =
                fps_from_delta(now.saturating_duration_since(previous).as_secs_f32());
        } else {
            state.actual_game_render_fps = state.config.render_fps;
        }
    }

    pub fn mark_game_frame_cache_invalid(&self) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        state.cache_valid = false;
        state.render_due = true;
    }

    pub fn mark_host_presented_cached_frame(&self) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        if state.config.presentation.hold_last_game_frame {
            state.render_due = false;
        }
    }

    pub fn force_single_simulation_tick(&self, delta_seconds: f32) {
        let mut state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        state.pending_simulation_ticks = 1;
        state.simulation_delta_seconds = delta_seconds.max(0.0);
        state.host_delta_seconds = state.simulation_delta_seconds;
    }

    pub fn snapshot(&self) -> RuntimeFrameClockSnapshot {
        let state = self
            .inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned");
        snapshot_from_state(&state)
    }

    pub fn host_delta_seconds(&self) -> f32 {
        self.inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned")
            .host_delta_seconds
    }

    pub fn simulation_delta_seconds(&self) -> f32 {
        self.inner
            .lock()
            .expect("runtime frame clock mutex should not be poisoned")
            .simulation_delta_seconds
    }
}

pub fn simulation_delta_seconds(runtime: &Runtime) -> f32 {
    runtime
        .resolve::<RuntimeFrameClockService>()
        .map(|clock| clock.simulation_delta_seconds())
        .unwrap_or(1.0 / 60.0)
}

pub fn host_delta_seconds(runtime: &Runtime) -> f32 {
    runtime
        .resolve::<RuntimeFrameClockService>()
        .map(|clock| clock.host_delta_seconds())
        .unwrap_or(1.0 / 60.0)
}

struct TickConsumption {
    ticks: u32,
    dropped_debt: Duration,
}

fn consume_ticks(accumulator: &mut Duration, step: Duration, max_ticks: u32) -> TickConsumption {
    let mut ticks = 0;
    while *accumulator >= step && ticks < max_ticks {
        *accumulator -= step;
        ticks += 1;
    }
    let dropped_debt = if ticks == max_ticks && *accumulator >= step {
        let dropped = *accumulator;
        *accumulator = Duration::ZERO;
        dropped
    } else {
        Duration::ZERO
    };
    TickConsumption {
        ticks,
        dropped_debt,
    }
}

fn snapshot_from_state(state: &RuntimeFrameClockState) -> RuntimeFrameClockSnapshot {
    RuntimeFrameClockSnapshot {
        strategy: state.config.strategy,
        host_frame_index: state.host_frame_index,
        simulation_tick_index: state.simulation_tick_index,
        game_render_frame_index: state.game_render_frame_index,
        host_delta_seconds: state.host_delta_seconds,
        simulation_delta_seconds: state.simulation_delta_seconds,
        target_simulation_fps: state.config.simulation_fps,
        target_render_fps: state.config.render_fps,
        actual_host_fps: state.actual_host_fps,
        actual_game_render_fps: state.actual_game_render_fps,
        scheduled_simulation_ticks: state.scheduled_simulation_ticks,
        consumed_simulation_ticks: state.consumed_simulation_ticks,
        pending_simulation_ticks: state.pending_simulation_ticks,
        dropped_simulation_debt_seconds: state.dropped_simulation_debt_seconds,
        should_render_game_frame: state.render_due,
        holding_cached_game_frame: state.config.presentation.hold_last_game_frame
            && state.cache_valid
            && !state.render_due,
    }
}

fn fps_from_delta(delta_seconds: f32) -> f32 {
    if delta_seconds.is_finite() && delta_seconds > 0.0 {
        1.0 / delta_seconds
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResolvedFramePresentationConfig, ResolvedPresentationLayerMode};

    fn config(
        strategy: ResolvedFrameClockStrategy,
        simulation_fps: f32,
        render_fps: f32,
        max_catch_up_ticks: u32,
    ) -> ResolvedFrameClockConfig {
        ResolvedFrameClockConfig {
            strategy,
            simulation_fps,
            render_fps,
            max_catch_up_ticks,
            clamp_frame_delta_seconds: 0.25,
            presentation: ResolvedFramePresentationConfig {
                cache_game_frame: true,
                hold_last_game_frame: true,
                game_ui: ResolvedPresentationLayerMode::Cached,
                devtools_live: true,
                editor_live: true,
                debug_overlay_live: true,
            },
        }
    }

    #[test]
    fn fixed_simulation_consumes_catch_up_ticks_with_fixed_dt() {
        let clock = RuntimeFrameClockService::default();
        clock.configure(config(
            ResolvedFrameClockStrategy::FixedSimulationSampledRender,
            60.0,
            12.0,
            5,
        ));
        let start = Instant::now();

        clock.begin_host_frame(start);
        clock.mark_game_frame_rendered();
        clock.begin_host_frame(start + Duration::from_millis(100));

        let scheduled = clock.snapshot();
        assert_eq!(scheduled.scheduled_simulation_ticks, 5);
        assert_eq!(scheduled.pending_simulation_ticks, 5);
        let ticks = clock.take_pending_simulation_ticks();
        assert_eq!(ticks.len(), 5);
        assert!(ticks.iter().all(|dt| (*dt - (1.0 / 60.0)).abs() < 0.0001));
        let consumed = clock.snapshot();
        assert_eq!(consumed.scheduled_simulation_ticks, 5);
        assert_eq!(consumed.consumed_simulation_ticks, 5);
        assert_eq!(consumed.pending_simulation_ticks, 0);
        assert!(clock.should_render_game_frame());
    }

    #[test]
    fn fixed_simulation_reports_dropped_debt_when_catch_up_saturates() {
        let clock = RuntimeFrameClockService::default();
        clock.configure(config(
            ResolvedFrameClockStrategy::FixedSimulationSampledRender,
            60.0,
            60.0,
            1,
        ));
        let start = Instant::now();

        clock.begin_host_frame(start);
        clock.begin_host_frame(start + Duration::from_millis(100));

        let snapshot = clock.snapshot();
        assert_eq!(snapshot.scheduled_simulation_ticks, 1);
        assert!(snapshot.dropped_simulation_debt_seconds > 0.0);
    }

    #[test]
    fn game_render_fps_is_measured_from_render_intervals() {
        let clock = RuntimeFrameClockService::default();
        let start = Instant::now();

        clock.mark_game_frame_rendered_at(start);
        clock.mark_game_frame_rendered_at(start + Duration::from_millis(100));

        let snapshot = clock.snapshot();
        assert!((snapshot.actual_game_render_fps - 10.0).abs() < 0.01);
    }

    #[test]
    fn fixed_update_and_render_uses_simulation_fps_for_dt() {
        let clock = RuntimeFrameClockService::default();
        clock.configure(config(
            ResolvedFrameClockStrategy::FixedUpdateAndRender,
            12.0,
            12.0,
            3,
        ));
        let start = Instant::now();

        clock.begin_host_frame(start);
        clock.mark_game_frame_rendered();
        clock.begin_host_frame(start + Duration::from_millis(84));

        let ticks = clock.take_pending_simulation_ticks();
        assert_eq!(ticks.len(), 1);
        assert!((ticks[0] - (1.0 / 12.0)).abs() < 0.0001);
    }

    #[test]
    fn sampled_render_holds_cached_frame_between_render_samples() {
        let clock = RuntimeFrameClockService::default();
        clock.configure(config(
            ResolvedFrameClockStrategy::FixedSimulationSampledRender,
            60.0,
            12.0,
            5,
        ));
        let start = Instant::now();

        clock.begin_host_frame(start);
        clock.mark_game_frame_rendered();
        clock.mark_host_presented_cached_frame();
        clock.begin_host_frame(start + Duration::from_millis(16));

        let snapshot = clock.snapshot();
        assert!(!snapshot.should_render_game_frame);
        assert!(snapshot.holding_cached_game_frame);
    }

    #[test]
    fn realtime_sampled_render_uses_host_delta_for_simulation() {
        let clock = RuntimeFrameClockService::default();
        clock.configure(config(
            ResolvedFrameClockStrategy::RealtimeUpdateSampledRender,
            60.0,
            12.0,
            5,
        ));
        let start = Instant::now();

        clock.begin_host_frame(start);
        clock.mark_game_frame_rendered();
        clock.begin_host_frame(start + Duration::from_millis(20));

        let ticks = clock.take_pending_simulation_ticks();
        assert_eq!(ticks, vec![0.020]);
        assert!(!clock.should_render_game_frame());
    }
}
