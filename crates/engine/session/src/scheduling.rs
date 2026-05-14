use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amigo_runtime::EngineSchedulerMode;

#[derive(Debug, Clone)]
pub struct ResolvedSchedulingOverride {
    pub target: String,
    pub lane: Option<String>,
    pub priority: Option<String>,
    pub parallelism: Option<String>,
    pub allow_frame_latency: Option<bool>,
    pub quality_scale: Option<f32>,
    pub budget_ms: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SchedulingOverrideReport {
    pub target: String,
    pub domain: String,
    pub matched: bool,
    pub resolved_target: Option<String>,
    pub quality_scale: Option<f32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSchedulingConfig {
    pub mode: EngineSchedulerMode,
    pub max_workers: usize,
    pub deterministic: bool,
    pub allow_frame_latency: bool,
    pub overrides: Vec<ResolvedSchedulingOverride>,
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
pub struct SchedulingFrameStats {
    pub mode: EngineSchedulerMode,
    pub particle_mode: String,
    pub particle_update_ms: f32,
    pub render_prepare_ms: f32,
    pub worker_jobs_submitted: usize,
    pub worker_jobs_completed: usize,
    pub particle_live_count: usize,
    pub particle_spawned_count: usize,
    pub worker_waited_this_frame: bool,
    pub particle_job_in_flight: bool,
    pub reused_previous_particle_frame: bool,
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

pub struct RuntimeSchedulingService {
    config: Mutex<ResolvedSchedulingConfig>,
    stats: Mutex<SchedulingFrameStats>,
    override_reports: Mutex<Vec<SchedulingOverrideReport>>,
    particle_job_in_flight: AtomicBool,
    particle_budget_scale: Mutex<f32>,
}

impl Default for RuntimeSchedulingService {
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

impl RuntimeSchedulingService {
    pub fn config(&self) -> ResolvedSchedulingConfig {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .clone()
    }

    pub fn set_config(&self, config: ResolvedSchedulingConfig) {
        *self
            .config
            .lock()
            .expect("scheduling config mutex should not be poisoned") = config;
    }

    pub fn mode(&self) -> EngineSchedulerMode {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .mode
    }

    pub fn set_mode(&self, mode: EngineSchedulerMode) {
        self.config
            .lock()
            .expect("scheduling config mutex should not be poisoned")
            .mode = mode;
        self.stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned")
            .mode = mode;
    }

    pub fn stats(&self) -> SchedulingFrameStats {
        self.stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned")
            .clone()
    }

    pub fn set_stats(&self, stats: SchedulingFrameStats) {
        *self
            .stats
            .lock()
            .expect("scheduling stats mutex should not be poisoned") = stats;
    }

    pub fn set_override_reports(&self, reports: Vec<SchedulingOverrideReport>) {
        *self
            .override_reports
            .lock()
            .expect("scheduling override report mutex should not be poisoned") = reports;
    }

    pub fn override_reports(&self) -> Vec<SchedulingOverrideReport> {
        self.override_reports
            .lock()
            .expect("scheduling override report mutex should not be poisoned")
            .clone()
    }

    pub fn try_begin_particle_job(&self) -> bool {
        self.particle_job_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn finish_particle_job(&self) {
        self.particle_job_in_flight.store(false, Ordering::Release);
    }

    pub fn particle_job_in_flight(&self) -> bool {
        self.particle_job_in_flight.load(Ordering::Acquire)
    }

    pub fn particle_budget_scale(&self) -> f32 {
        *self
            .particle_budget_scale
            .lock()
            .expect("particle budget scale mutex should not be poisoned")
    }

    pub fn set_particle_budget_scale(&self, scale: f32) {
        *self
            .particle_budget_scale
            .lock()
            .expect("particle budget scale mutex should not be poisoned") = scale.max(0.0);
    }
}
