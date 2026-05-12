use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    EventPipelineStepSceneCommand, SceneCommand, SceneEvent, SceneEventQueue, SceneService,
    format_scene_command,
};

use crate::{EventPipeline, EventPipelineService, EventPipelineStep};

pub struct EventPipelineSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub event_pipeline_service: &'a EventPipelineService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct EventPipelineSceneCommandOutcome {
    pub id: String,
    pub source_mod: String,
}

pub fn can_handle_event_pipeline_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueEventPipeline { .. })
}

pub fn handle_event_pipeline_scene_command(
    ctx: EventPipelineSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<EventPipelineSceneCommandOutcome> {
    match command {
        SceneCommand::QueueEventPipeline { command } => {
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.event_pipeline_service.queue(EventPipeline {
                id: command.id.clone(),
                topic: command.topic,
                steps: command
                    .steps
                    .into_iter()
                    .map(event_pipeline_step_from_scene_command)
                    .collect(),
            });
            ctx.scene_event_queue
                .publish(SceneEvent::EventPipelineQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name,
                });
            Ok(EventPipelineSceneCommandOutcome {
                id: command.id,
                source_mod: command.source_mod,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "event-pipeline cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn event_pipeline_step_from_scene_command(
    step: EventPipelineStepSceneCommand,
) -> EventPipelineStep {
    match step {
        EventPipelineStepSceneCommand::PlayAudio { clip } => EventPipelineStep::PlayAudio { clip },
        EventPipelineStepSceneCommand::SetState { key, value } => {
            EventPipelineStep::SetState { key, value }
        }
        EventPipelineStepSceneCommand::IncrementState { key, by } => {
            EventPipelineStep::IncrementState { key, by }
        }
        EventPipelineStepSceneCommand::ShowUi { path } => EventPipelineStep::ShowUi { path },
        EventPipelineStepSceneCommand::HideUi { path } => EventPipelineStep::HideUi { path },
        EventPipelineStepSceneCommand::BurstParticles { emitter, count } => {
            EventPipelineStep::BurstParticles { emitter, count }
        }
        EventPipelineStepSceneCommand::TransitionScene { scene } => {
            EventPipelineStep::TransitionScene { scene }
        }
        EventPipelineStepSceneCommand::EmitEvent { topic, payload } => {
            EventPipelineStep::EmitEvent { topic, payload }
        }
        EventPipelineStepSceneCommand::Script { function } => EventPipelineStep::Script { function },
    }
}
