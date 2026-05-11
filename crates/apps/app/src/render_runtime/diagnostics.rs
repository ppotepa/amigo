use std::sync::Mutex;

use amigo_render_api::{FrameCompositionPlan, FrameGraph, RenderCompositionDiagnostics};

#[derive(Debug, Default)]
pub(crate) struct RenderCompositionDiagnosticsService {
    inner: Mutex<RenderCompositionDiagnostics>,
}

impl RenderCompositionDiagnosticsService {
    pub(crate) fn set(&self, plan: &FrameCompositionPlan, graph: &FrameGraph) {
        *self
            .inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned") =
            RenderCompositionDiagnostics::from_plan_and_graph(plan, graph);
    }

    pub(crate) fn snapshot(&self) -> RenderCompositionDiagnostics {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .clone()
    }
}
