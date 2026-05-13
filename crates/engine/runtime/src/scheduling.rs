#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineSchedulerMode {
    SingleThread,
    Auto,
    Hybrid,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineLane {
    Main,
    Simulation,
    RenderPrepare,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadPolicy {
    MainOnly,
    WorkerAllowed,
    BackgroundOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parallelism {
    None,
    PerSystem,
    PerEmitter,
    PerLayer,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPriority {
    Background,
    Low,
    Normal,
    Foreground,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineSchedulingConfig {
    pub mode: EngineSchedulerMode,
    pub max_workers: usize,
    pub deterministic: bool,
    pub allow_frame_latency: bool,
}

impl Default for EngineSchedulingConfig {
    fn default() -> Self {
        Self {
            mode: EngineSchedulerMode::SingleThread,
            max_workers: 0,
            deterministic: true,
            allow_frame_latency: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulingDescriptor {
    pub id: &'static str,
    pub lane: EngineLane,
    pub thread_policy: ThreadPolicy,
    pub parallelism: Parallelism,
    pub priority: SchedulingPriority,
    pub allow_frame_latency: bool,
}

impl SchedulingDescriptor {
    pub fn main_only(id: &'static str) -> Self {
        Self {
            id,
            lane: EngineLane::Main,
            thread_policy: ThreadPolicy::MainOnly,
            parallelism: Parallelism::None,
            priority: SchedulingPriority::Normal,
            allow_frame_latency: false,
        }
    }
}

