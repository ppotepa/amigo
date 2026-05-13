use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use amigo_scripting_api::RunLogService;

const MAX_EMERGENCY_NOTICES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyNoticeLevel {
    Warning,
    Error,
}

impl EmergencyNoticeLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmergencyNotice {
    pub level: EmergencyNoticeLevel,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct EmergencyNoticeService {
    inner: Mutex<EmergencyNoticeServiceInner>,
}

#[derive(Debug, Default)]
struct EmergencyNoticeServiceInner {
    notices: VecDeque<EmergencyNotice>,
    run_log: Option<Arc<RunLogService>>,
}

impl EmergencyNoticeService {
    pub fn attach_run_log(&self, run_log: Arc<RunLogService>) {
        self.inner
            .lock()
            .expect("emergency notice mutex should not be poisoned")
            .run_log = Some(run_log);
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.push(EmergencyNoticeLevel::Warning, message);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.push(EmergencyNoticeLevel::Error, message);
    }

    pub fn push(&self, level: EmergencyNoticeLevel, message: impl Into<String>) {
        let message = message.into();
        let mut inner = self
            .inner
            .lock()
            .expect("emergency notice mutex should not be poisoned");

        if inner
            .notices
            .back()
            .is_some_and(|notice| notice.level == level && notice.message == message)
        {
            return;
        }

        if let Some(run_log) = &inner.run_log {
            run_log.write_runtime(format!("{}: {message}", level.as_str()));
        }

        inner.notices.push_back(EmergencyNotice { level, message });
        while inner.notices.len() > MAX_EMERGENCY_NOTICES {
            inner.notices.pop_front();
        }
    }

    pub fn snapshot(&self) -> Vec<EmergencyNotice> {
        self.inner
            .lock()
            .expect("emergency notice mutex should not be poisoned")
            .notices
            .iter()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{EmergencyNoticeLevel, EmergencyNoticeService};

    #[test]
    fn keeps_latest_warning_and_error_notices() {
        let service = EmergencyNoticeService::default();
        service.warn("missing optional texture");
        service.error("failed to decode base_albedo.png");

        let notices = service.snapshot();
        assert_eq!(notices.len(), 2);
        assert_eq!(notices[0].level, EmergencyNoticeLevel::Warning);
        assert_eq!(notices[1].level, EmergencyNoticeLevel::Error);
    }
}
