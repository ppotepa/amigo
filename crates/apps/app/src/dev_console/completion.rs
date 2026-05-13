use super::registry::ConsoleCommandRegistry;
use amigo_devtools::{
    ConsoleCompletionState as EngineConsoleCompletionState,
};

pub(crate) use amigo_devtools::{
    ConsoleCompletionKind, ConsoleCompletionSnapshot, ConsoleCompletionSuggestion,
};

#[derive(Debug, Default)]
pub(crate) struct ConsoleCompletionState {
    inner: EngineConsoleCompletionState,
}

impl ConsoleCompletionState {
    pub(crate) fn snapshot(&self) -> Option<ConsoleCompletionSnapshot> {
        self.inner.snapshot()
    }

    pub(crate) fn clear(&self) {
        self.inner.clear()
    }

    pub(crate) fn refresh(&self, input: &str, registry: &ConsoleCommandRegistry) {
        self.inner.refresh(input, &registry.descriptors());
    }

    pub(crate) fn select_next(&self) -> bool {
        self.select_delta(1)
    }

    pub(crate) fn select_previous(&self) -> bool {
        self.select_delta(-1)
    }

    pub(crate) fn accept_tab(&self, input: &str) -> Option<String> {
        let snapshot = self.snapshot()?;
        amigo_devtools::accept_completion_tab(input, &snapshot)
    }

    fn select_delta(&self, delta: isize) -> bool {
        if delta > 0 {
            self.inner.select_next()
        } else {
            self.inner.select_previous()
        }
    }
}

pub(crate) fn compute_console_completion(
    input: &str,
    registry: &ConsoleCommandRegistry,
) -> Option<ConsoleCompletionSnapshot> {
    let descriptors = registry.descriptors();
    amigo_devtools::compute_console_completion_from_descriptors(input, &descriptors)
}



