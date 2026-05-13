use crate::RuntimeSession;

/// Result object returned by host-specific bootstrap adapters.
pub struct RuntimeSessionBootstrap<TSummary> {
    session: RuntimeSession,
    summary: TSummary,
}

impl<TSummary> RuntimeSessionBootstrap<TSummary> {
    pub fn new(session: RuntimeSession, summary: TSummary) -> Self {
        Self { session, summary }
    }

    pub fn session(&self) -> &RuntimeSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut RuntimeSession {
        &mut self.session
    }

    pub fn summary(&self) -> &TSummary {
        &self.summary
    }

    pub fn into_parts(self) -> (RuntimeSession, TSummary) {
        (self.session, self.summary)
    }
}

