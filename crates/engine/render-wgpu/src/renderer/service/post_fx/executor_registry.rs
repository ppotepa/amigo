use std::collections::BTreeMap;
use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};

use super::executor::{WgpuPostFxExecutionContext, WgpuPostFxExecutor};
use crate::renderer::service::WgpuSceneRenderer;

#[derive(Default)]
pub(crate) struct WgpuPostFxExecutorRegistry {
    executors: BTreeMap<&'static str, Arc<dyn WgpuPostFxExecutor>>,
}

impl WgpuPostFxExecutorRegistry {
    pub(crate) fn register(&mut self, executor: impl WgpuPostFxExecutor + 'static) {
        self.executors
            .insert(executor.executor_id(), Arc::new(executor));
    }

    pub(crate) fn executor(
        &self,
        executor_id: &str,
        effect_kind: &str,
    ) -> AmigoResult<Arc<dyn WgpuPostFxExecutor>> {
        self.executors.get(executor_id).cloned().ok_or_else(|| {
            AmigoError::Message(format!(
                "post-fx executor {} is not registered for feature {}",
                executor_id, effect_kind
            ))
        })
    }
}
