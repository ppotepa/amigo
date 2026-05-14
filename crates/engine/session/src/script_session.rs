use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSessionLifecycleState {
    Idle,
    DispatchingCommand,
    DispatchedCommand,
    Error,
}

impl Default for ScriptSessionLifecycleState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub struct ScriptSession {
    state: ScriptSessionLifecycleState,
    last_command: Option<String>,
    command_count: u64,
    last_error: Option<String>,
}

impl Default for ScriptSession {
    fn default() -> Self {
        Self {
            state: ScriptSessionLifecycleState::Idle,
            last_command: None,
            command_count: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptSessionService {
    inner: Arc<Mutex<ScriptSession>>,
}

impl ScriptSessionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> ScriptSession {
        self.with_session(Clone::clone)
    }

    pub fn lifecycle_state(&self) -> ScriptSessionLifecycleState {
        self.with_session(ScriptSession::lifecycle_state)
    }

    pub fn script_dispatch_summary(&self) -> ScriptCommandDispatchSummary {
        self.with_session(ScriptSession::script_dispatch_summary)
    }

    pub fn begin_script_command_dispatch(
        &self,
        command: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.with_session_mut(|session| session.begin_script_command_dispatch(command))
    }

    pub fn complete_script_command_dispatch(&self) -> ScriptCommandDispatchSummary {
        self.with_session_mut(ScriptSession::complete_script_command_dispatch)
    }

    pub fn mark_script_dispatch_error(
        &self,
        command: impl Into<String>,
        error: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.with_session_mut(|session| session.mark_script_dispatch_error(command, error))
    }

    fn with_session<T>(&self, f: impl FnOnce(&ScriptSession) -> T) -> T {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        f(&guard)
    }

    fn with_session_mut<T>(&self, f: impl FnOnce(&mut ScriptSession) -> T) -> T {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        f(&mut guard)
    }
}

impl ScriptSession {
    pub fn lifecycle_state(&self) -> ScriptSessionLifecycleState {
        self.state
    }

    pub fn script_dispatch_summary(&self) -> ScriptCommandDispatchSummary {
        ScriptCommandDispatchSummary {
            state: self.state,
            last_command: self.last_command.clone(),
            command_count: self.command_count,
            last_error: self.last_error.clone(),
        }
    }

    pub fn begin_script_command_dispatch(
        &mut self,
        command: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.state = ScriptSessionLifecycleState::DispatchingCommand;
        self.last_command = Some(command.into());
        self.last_error = None;
        self.script_dispatch_summary()
    }

    pub fn complete_script_command_dispatch(&mut self) -> ScriptCommandDispatchSummary {
        self.state = ScriptSessionLifecycleState::DispatchedCommand;
        self.command_count = self.command_count.saturating_add(1);
        self.script_dispatch_summary()
    }

    pub fn mark_script_dispatch_error(
        &mut self,
        command: impl Into<String>,
        error: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.state = ScriptSessionLifecycleState::Error;
        self.last_command = Some(command.into());
        self.last_error = Some(error.into());
        self.script_dispatch_summary()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptCommandDispatchSummary {
    pub state: ScriptSessionLifecycleState,
    pub last_command: Option<String>,
    pub command_count: u64,
    pub last_error: Option<String>,
}
