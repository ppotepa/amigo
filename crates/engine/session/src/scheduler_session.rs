use amigo_runtime::SystemPhase;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerSessionLifecycleState {
    Idle,
    RunningPhase,
    CompletedPhase,
    Error,
}

impl Default for SchedulerSessionLifecycleState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerSession {
    state: SchedulerSessionLifecycleState,
    phase: Option<SystemPhase>,
    phase_runs: u64,
    last_error: Option<String>,
}

impl Default for SchedulerSession {
    fn default() -> Self {
        Self {
            state: SchedulerSessionLifecycleState::Idle,
            phase: None,
            phase_runs: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SchedulerSessionService {
    inner: Arc<Mutex<SchedulerSession>>,
}

impl SchedulerSessionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> SchedulerSession {
        self.with_session(Clone::clone)
    }

    pub fn lifecycle_state(&self) -> SchedulerSessionLifecycleState {
        self.with_session(SchedulerSession::lifecycle_state)
    }

    pub fn scheduler_summary(&self) -> SchedulerPhaseSummary {
        self.with_session(SchedulerSession::scheduler_summary)
    }

    pub fn begin_system_phase(&self, phase: SystemPhase) -> SchedulerPhaseSummary {
        self.with_session_mut(|session| session.begin_system_phase(phase))
    }

    pub fn complete_system_phase(&self, phase: SystemPhase) -> SchedulerPhaseSummary {
        self.with_session_mut(|session| session.complete_system_phase(phase))
    }

    pub fn mark_error(&self, phase: SystemPhase, error: impl Into<String>) -> SchedulerPhaseSummary {
        self.with_session_mut(|session| session.mark_error(phase, error))
    }

    fn with_session<T>(&self, f: impl FnOnce(&SchedulerSession) -> T) -> T {
        let guard = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
        f(&guard)
    }

    fn with_session_mut<T>(&self, f: impl FnOnce(&mut SchedulerSession) -> T) -> T {
        let mut guard = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
        f(&mut guard)
    }
}

impl SchedulerSession {
    pub fn lifecycle_state(&self) -> SchedulerSessionLifecycleState {
        self.state
    }

    pub fn scheduler_summary(&self) -> SchedulerPhaseSummary {
        SchedulerPhaseSummary {
            state: self.state,
            phase: phase_name(self.phase),
            phase_runs: self.phase_runs,
            last_error: self.last_error.clone(),
        }
    }

    pub fn begin_system_phase(&mut self, phase: SystemPhase) -> SchedulerPhaseSummary {
        self.state = SchedulerSessionLifecycleState::RunningPhase;
        self.phase = Some(phase);
        self.phase_runs = self.phase_runs.saturating_add(1);
        self.last_error = None;
        self.scheduler_summary()
    }

    pub fn complete_system_phase(&mut self, phase: SystemPhase) -> SchedulerPhaseSummary {
        if self.phase == Some(phase) {
            self.state = SchedulerSessionLifecycleState::CompletedPhase;
        } else {
            self.state = SchedulerSessionLifecycleState::Error;
            self.last_error = Some(format!(
                "phase mismatch: expected {:?} completed {:?}",
                phase_name(self.phase),
                phase_name(Some(phase)),
            ));
        }

        self.scheduler_summary()
    }

    pub fn mark_error(&mut self, phase: SystemPhase, error: impl Into<String>) -> SchedulerPhaseSummary {
        self.state = SchedulerSessionLifecycleState::Error;
        self.phase = Some(phase);
        self.last_error = Some(error.into());
        self.scheduler_summary()
    }
}

fn phase_name(phase: Option<SystemPhase>) -> String {
    match phase {
        Some(SystemPhase::PreUpdate) => "pre_update".to_owned(),
        Some(SystemPhase::Update) => "update".to_owned(),
        Some(SystemPhase::PostUpdate) => "post_update".to_owned(),
        Some(other) => format!("{other:?}").to_lowercase(),
        None => "idle".to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerPhaseSummary {
    pub state: SchedulerSessionLifecycleState,
    pub phase: String,
    pub phase_runs: u64,
    pub last_error: Option<String>,
}

