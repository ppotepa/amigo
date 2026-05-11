use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amigo_runtime::EngineSchedulerMode;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSchedulingOverride {
    pub(crate) target: String,
    pub(crate) lane: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) parallelism: Option<String>,
    pub(crate) allow_frame_latency: Option<bool>,
    pub(crate) quality_scale: Option<f32>,
    pub(crate) budget_ms: Option<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct SchedulingOverrideReport {
    pub(crate) target: String,
    pub(crate) domain: String,
    pub(crate) matched: bool,
    pub(crate) resolved_target: Option<String>,
    pub(crate) quality_scale: Option<f32>,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSchedulingConfig {
    pub(crate) mode: EngineSchedulerMode,
    pub(crate) max_workers: usize,
    pub(crate) deterministic: bool,
    pub(crate) allow_frame_latency: bool,
    pub(crate) overrides: Vec<ResolvedSchedulingOverride>,
}

impl Default for ResolvedSchedulingConfig {
    fn default() -> Self {
        Self {
            mode: EngineSchedulerMode::SingleThread,
            max_workers: 0,
            deterministic: true,
            allow_frame_latency: false,
            overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SchedulingFrameStats {
    pub(crate) mode: EngineSchedulerMode,
    pub(crate) particle_mode: String,
    pub(crate) particle_update_ms: f32,
    pub(crate) render_prepare_ms: f32,
    pub(crate) worker_jobs_submitted: usize,
    pub(crate) worker_jobs_completed: usize,
    pub(crate) particle_live_count: usize,
    pub(crate) particle_spawned_count: usize,
    pub(crate) worker_waited_this_frame: bool,
    pub(crate) particle_job_in_flight: bool,
    pub(crate) reused_previous_particle_frame: bool,
}

impl Default for SchedulingFrameStats {
    fn default() -> Self {
        Self {
            mode: EngineSchedulerMode::SingleThread,
            particle_mode: "legacy".to_owned(),
            particle_update_ms: 0.0,
            render_prepare_ms: 0.0,
            worker_jobs_submitted: 0,
            worker_jobs_completed: 0,
            particle_live_count: 0,
            particle_spawned_count: 0,
            worker_waited_this_frame: false,
            particle_job_in_flight: false,
            reused_previous_particle_frame: false,
        }
    }
}

pub(crate) struct AppSchedulingService {
    config: Mutex<ResolvedSchedulingConfig>,
    stats: Mutex<SchedulingFrameStats>,
    override_reports: Mutex<Vec<SchedulingOverrideReport>>,
    particle_job_in_flight: AtomicBool,
    particle_budget_scale: Mutex<f32>,
}

impl Default for AppSchedulingService {
    fn default() -> Self {
        Self {
            config: Mutex::new(ResolvedSchedulingConfig::default()),
            stats: Mutex::new(SchedulingFrameStats::default()),
            override_reports: Mutex::new(Vec::new()),
            particle_job_in_flight: AtomicBool::new(false),
            particle_budget_scale: Mutex::new(1.0),
        }
    }
}

impl AppSchedulingService {
    pub(crate) fn config(&self) -> ResolvedSchedulingConfig {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .clone()
    }

    pub(crate) fn set_config(&self, config: ResolvedSchedulingConfig) {
        *self
            .config
            .lock()
            .expect("scheduling config mutex should not be poisoned") = config;
    }

    pub(crate) fn mode(&self) -> EngineSchedulerMode {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .mode
    }

    pub(crate) fn set_mode(&self, mode: EngineSchedulerMode) {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .mode = mode;
        self.stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned")
            .mode = mode;
    }

    pub(crate) fn stats(&self) -> SchedulingFrameStats {
        self.stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned")
            .clone()
    }

    pub(crate) fn set_stats(&self, stats: SchedulingFrameStats) {
        *self
            .stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned") = stats;
    }

    pub(crate) fn set_override_reports(&self, reports: Vec<SchedulingOverrideReport>) {
        *self
            .override_reports
            .lock()
            .expect("scheduling override report mutex should not be poisoned") = reports;
    }

    pub(crate) fn override_reports(&self) -> Vec<SchedulingOverrideReport> {
        self.override_reports
            .lock()
            .expect("scheduling override report mutex should not be poisoned")
            .clone()
    }

    pub(crate) fn try_begin_particle_job(&self) -> bool {
        self.particle_job_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn finish_particle_job(&self) {
        self.particle_job_in_flight.store(false, Ordering::Release);
    }

    pub(crate) fn particle_job_in_flight(&self) -> bool {
        self.particle_job_in_flight.load(Ordering::Acquire)
    }

    pub(crate) fn particle_budget_scale(&self) -> f32 {
        *self
            .particle_budget_scale
            .lock()
            .expect("particle budget scale mutex should not be poisoned")
    }

    pub(crate) fn set_particle_budget_scale(&self, scale: f32) {
        *self
            .particle_budget_scale
            .lock()
            .expect("particle budget scale mutex should not be poisoned") = scale.max(0.0);
    }
}
