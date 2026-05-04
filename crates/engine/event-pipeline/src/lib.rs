//! Queued event routing for cross-system gameplay communication.
//! It collects named events and exposes dispatch state to runtime systems and scripts.

use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, PartialEq)]
pub struct EventPipeline {
    pub id: String,
    pub topic: String,
    pub steps: Vec<EventPipelineStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventPipelineStep {
    PlayAudio { clip: String },
    SetState { key: String, value: String },
    IncrementState { key: String, by: f64 },
    ShowUi { path: String },
    HideUi { path: String },
    BurstParticles { emitter: String, count: usize },
    TransitionScene { scene: String },
    EmitEvent { topic: String, payload: Vec<String> },
    Script { function: String },
}

#[derive(Debug, Default)]
pub struct EventPipelineService {
    pipelines: Mutex<Vec<EventPipeline>>,
}

impl EventPipelineService {
    pub fn queue(&self, pipeline: EventPipeline) {
        let mut pipelines = self
            .pipelines
            .lock()
            .expect("event pipeline service mutex should not be poisoned");
        if let Some(existing) = pipelines
            .iter_mut()
            .find(|existing| existing.id == pipeline.id)
        {
            *existing = pipeline;
        } else {
            pipelines.push(pipeline);
        }
    }

    pub fn pipelines(&self) -> Vec<EventPipeline> {
        self.pipelines
            .lock()
            .expect("event pipeline service mutex should not be poisoned")
            .clone()
    }

    pub fn pipelines_for_topic(&self, topic: &str) -> Vec<EventPipeline> {
        self.pipelines()
            .into_iter()
            .filter(|pipeline| pipeline.topic == topic)
            .collect()
    }

    pub fn clear(&self) {
        self.pipelines
            .lock()
            .expect("event pipeline service mutex should not be poisoned")
            .clear();
    }
}

pub struct EventPipelinePlugin;

impl RuntimePlugin for EventPipelinePlugin {
    fn name(&self) -> &'static str {
        "amigo-event-pipeline"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(EventPipelineService::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_filters_pipelines_by_topic() {
        let service = EventPipelineService::default();
        service.queue(EventPipeline {
            id: "hit".to_owned(),
            topic: "collision.hit".to_owned(),
            steps: vec![EventPipelineStep::IncrementState {
                key: "score".to_owned(),
                by: 100.0,
            }],
        });
        service.queue(EventPipeline {
            id: "other".to_owned(),
            topic: "ui.click".to_owned(),
            steps: Vec::new(),
        });

        assert_eq!(service.pipelines_for_topic("collision.hit").len(), 1);
        assert!(service.pipelines_for_topic("missing").is_empty());
    }
}
