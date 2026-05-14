use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSessionLifecycleState {
    Idle,
    Extracting,
    Extracted,
    Composing,
    Composed,
    GraphBuilt,
    Submitted,
    Presented,
    Error,
}

impl Default for RenderSessionLifecycleState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub struct RenderSession {
    lifecycle_state: RenderSessionLifecycleState,
    frame_index: u64,
    last_error: Option<String>,
}

impl Default for RenderSession {
    fn default() -> Self {
        Self {
            lifecycle_state: RenderSessionLifecycleState::Idle,
            frame_index: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderSessionService {
    inner: Arc<Mutex<RenderSession>>,
}

impl RenderSessionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> RenderSession {
        self.with_session(Clone::clone)
    }

    pub fn lifecycle_state(&self) -> RenderSessionLifecycleState {
        self.with_session(RenderSession::lifecycle_state)
    }

    pub fn frame_index(&self) -> u64 {
        self.with_session(RenderSession::frame_index)
    }

    pub fn lifecycle_summary(&self) -> RenderFrameLifecycleSummary {
        self.with_session(RenderSession::lifecycle_summary)
    }

    pub fn begin_frame_extract(&self) -> RenderFrameLifecycleSummary {
        self.with_session_mut(RenderSession::begin_frame_extract)
    }

    pub fn complete_frame_extract(&self) -> RenderFrameSummary {
        self.with_session_mut(RenderSession::complete_frame_extract)
    }

    pub fn begin_composition(&self) -> RenderFrameLifecycleSummary {
        self.with_session_mut(RenderSession::begin_composition)
    }

    pub fn complete_composition(&self) -> RenderFrameSummary {
        self.with_session_mut(RenderSession::complete_composition)
    }

    pub fn complete_graph_build(&self) -> RenderFrameSummary {
        self.with_session_mut(RenderSession::complete_graph_build)
    }

    pub fn complete_submit(&self) -> RenderFrameSummary {
        self.with_session_mut(RenderSession::complete_submit)
    }

    pub fn complete_present(&self) -> RenderFrameSummary {
        self.with_session_mut(RenderSession::complete_present)
    }

    pub fn mark_error(&self, error: impl Into<String>) -> RenderFrameErrorSummary {
        self.with_session_mut(|session| session.mark_error(error))
    }

    fn with_session<T>(&self, f: impl FnOnce(&RenderSession) -> T) -> T {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        f(&guard)
    }

    fn with_session_mut<T>(&self, f: impl FnOnce(&mut RenderSession) -> T) -> T {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        f(&mut guard)
    }
}

impl RenderSession {
    pub fn lifecycle_state(&self) -> RenderSessionLifecycleState {
        self.lifecycle_state
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn lifecycle_summary(&self) -> RenderFrameLifecycleSummary {
        RenderFrameLifecycleSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            last_error: self.last_error.clone(),
        }
    }

    pub fn begin_frame_extract(&mut self) -> RenderFrameLifecycleSummary {
        self.frame_index = self.frame_index.saturating_add(1);
        self.lifecycle_state = RenderSessionLifecycleState::Extracting;
        self.last_error = None;
        self.lifecycle_summary()
    }

    pub fn complete_frame_extract(&mut self) -> RenderFrameSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Extracted;
        RenderFrameSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            stage: "frame_extract".to_owned(),
        }
    }

    pub fn begin_composition(&mut self) -> RenderFrameLifecycleSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Composing;
        self.lifecycle_summary()
    }

    pub fn complete_composition(&mut self) -> RenderFrameSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Composed;
        RenderFrameSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            stage: "composition".to_owned(),
        }
    }

    pub fn complete_graph_build(&mut self) -> RenderFrameSummary {
        self.lifecycle_state = RenderSessionLifecycleState::GraphBuilt;
        RenderFrameSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            stage: "graph_build".to_owned(),
        }
    }

    pub fn complete_submit(&mut self) -> RenderFrameSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Submitted;
        RenderFrameSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            stage: "submit".to_owned(),
        }
    }

    pub fn complete_present(&mut self) -> RenderFrameSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Presented;
        RenderFrameSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            stage: "present".to_owned(),
        }
    }

    pub fn mark_error(&mut self, error: impl Into<String>) -> RenderFrameErrorSummary {
        self.lifecycle_state = RenderSessionLifecycleState::Error;
        self.last_error = Some(error.into());
        RenderFrameErrorSummary {
            state: self.lifecycle_state,
            frame_index: self.frame_index,
            error: self.last_error.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameLifecycleSummary {
    pub state: RenderSessionLifecycleState,
    pub frame_index: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameSummary {
    pub state: RenderSessionLifecycleState,
    pub frame_index: u64,
    pub stage: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameErrorSummary {
    pub state: RenderSessionLifecycleState,
    pub frame_index: u64,
    pub error: String,
}
