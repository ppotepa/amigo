use std::sync::Mutex;

use amigo_render_api::{FrameCompositionPlan, FrameGraph, RenderCompositionDiagnostics};
use amigo_render_wgpu::WgpuFrameGraphExecutionMode;

#[derive(Debug, Default)]
pub(crate) struct RenderCompositionDiagnosticsService {
    inner: Mutex<RenderCompositionDiagnostics>,
}

#[derive(Debug)]
pub(crate) struct RenderCompositionRuntimeService {
    mode: Mutex<WgpuFrameGraphExecutionMode>,
}

impl Default for RenderCompositionRuntimeService {
    fn default() -> Self {
        Self {
            mode: Mutex::new(WgpuFrameGraphExecutionMode::LegacyComposite),
        }
    }
}

impl RenderCompositionRuntimeService {
    pub(crate) fn mode(&self) -> WgpuFrameGraphExecutionMode {
        *self
            .mode
            .lock()
            .expect("render composition runtime mutex should not be poisoned")
    }

    pub(crate) fn set_mode(&self, mode: WgpuFrameGraphExecutionMode) {
        *self
            .mode
            .lock()
            .expect("render composition runtime mutex should not be poisoned") = mode;
    }
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
