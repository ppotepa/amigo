use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectSource {
    ConsoleCommand,
    Rhai,
    Devtools,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectSubject {
    Selected,
    Entity { name: String },
    PostFxFrameItem { index: usize, label: Option<String> },
    RenderLayer { id: String },
    AuthoringNode { node_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectRequest {
    pub source: InspectSource,
    pub subject: InspectSubject,
    pub expression: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectRequestService {
    inner: Arc<Mutex<VecDeque<InspectRequest>>>,
}

impl InspectRequestService {
    pub fn request(&self, request: InspectRequest) {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(request);
        }
    }

    pub fn request_subject(&self, source: InspectSource, subject: InspectSubject) {
        self.request(InspectRequest {
            source,
            subject,
            expression: None,
        });
    }

    pub fn request_expression(
        &self,
        source: InspectSource,
        subject: InspectSubject,
        expression: impl Into<String>,
    ) {
        self.request(InspectRequest {
            source,
            subject,
            expression: Some(expression.into()),
        });
    }

    pub fn take_latest(&self) -> Option<InspectRequest> {
        let mut queue = self.inner.lock().ok()?;
        let latest = queue.pop_back();
        queue.clear();
        latest
    }

    pub fn drain(&self) -> Vec<InspectRequest> {
        let mut queue = match self.inner.lock() {
            Ok(queue) => queue,
            Err(_) => return Vec::new(),
        };
        queue.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .map(|queue| queue.is_empty())
            .unwrap_or(true)
    }
}
