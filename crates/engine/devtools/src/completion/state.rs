use std::sync::Mutex;

use crate::{ConsoleCommandDescriptor, ConsoleCommandSchema};

use super::{
    ConsoleCompletionContext, ConsoleCompletionEdit, ConsoleCompletionSnapshot,
    accept_completion_tab, compute_console_completion_from_descriptors,
};

#[derive(Debug, Default)]
struct ConsoleCompletionInner {
    snapshot: Option<ConsoleCompletionSnapshot>,
}

#[derive(Debug, Default)]
pub struct ConsoleCompletionState {
    inner: Mutex<ConsoleCompletionInner>,
}

impl ConsoleCompletionState {
    pub fn snapshot(&self) -> Option<ConsoleCompletionSnapshot> {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot
            .clone()
    }

    pub fn clear(&self) {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = None;
    }

    pub fn refresh(
        &self,
        input: &str,
        cursor_index: usize,
        descriptors: &[ConsoleCommandDescriptor],
        schemas: &[ConsoleCommandSchema],
        context: &ConsoleCompletionContext,
    ) {
        let snapshot = compute_console_completion_from_descriptors(
            input,
            cursor_index,
            descriptors,
            schemas,
            context,
        );
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = snapshot.filter(ConsoleCompletionSnapshot::is_active);
    }

    pub fn select_next(&self) -> bool {
        self.select_delta(1)
    }

    pub fn select_previous(&self) -> bool {
        self.select_delta(-1)
    }

    pub fn accept_tab(&self, input: &str, cursor_index: usize) -> Option<ConsoleCompletionEdit> {
        let snapshot = self.snapshot()?;
        if snapshot.input != input || snapshot.cursor_index != cursor_index {
            return None;
        }
        accept_completion_tab(input, &snapshot)
    }

    fn select_delta(&self, delta: isize) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("console completion mutex should not be poisoned");
        let Some(snapshot) = inner.snapshot.as_mut() else {
            return false;
        };
        if snapshot.suggestions.is_empty() {
            return false;
        }

        let len = snapshot.suggestions.len() as isize;
        let next = (snapshot.selected_index as isize + delta).rem_euclid(len);
        snapshot.selected_index = next as usize;
        true
    }
}
