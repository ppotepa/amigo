use super::contracts::{NprPipelineInput, NprPipelineStage};
use crate::NprRenderPacket;
use std::{collections::BTreeMap, sync::Arc};

pub trait NprPipeline: Send + Sync {
    fn id(&self) -> &'static str;
    fn stages(&self) -> &'static [NprPipelineStage];
    fn build(&self, input: NprPipelineInput<'_>) -> NprRenderPacket;
}

#[derive(Default)]
pub struct NprPipelineRegistry {
    pipelines: BTreeMap<String, Arc<dyn NprPipeline>>,
}

impl NprPipelineRegistry {
    pub fn register(&mut self, pipeline: Arc<dyn NprPipeline>) -> Result<(), String> {
        let id = pipeline.id().to_owned();
        if self.pipelines.contains_key(&id) {
            return Err(format!("NPR pipeline `{id}` is already registered"));
        }
        self.pipelines.insert(id, pipeline);
        Ok(())
    }

    pub fn pipeline(&self, id: &str) -> Option<Arc<dyn NprPipeline>> {
        self.pipelines.get(id).cloned()
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.pipelines.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MinimalInkPipeline, PencilAnimationPipeline};

    #[test]
    fn duplicate_registration_keeps_the_original_pipeline() {
        let mut registry = NprPipelineRegistry::default();
        registry.register(Arc::new(MinimalInkPipeline::default())).unwrap();
        assert!(registry.register(Arc::new(MinimalInkPipeline::default())).is_err());
        assert_eq!(registry.pipeline("minimal-ink").unwrap().id(), "minimal-ink");
        registry.register(Arc::new(PencilAnimationPipeline::default())).unwrap();
        assert_eq!(registry.ids().collect::<Vec<_>>(), vec!["minimal-ink", "pencil-animation"]);
    }
}
